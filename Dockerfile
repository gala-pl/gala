# syntax=docker/dockerfile:1

# Multi-stage build for Gala.
#
# Builder: Ubuntu focal (ships LLVM 10) + rustup with the pinned toolchain.
# LLVM 10 dev libraries are required by the inkwell (llvm10-0) codegen backend
# when the `classical` feature of gala-codegen-classical is enabled.
# Runtime: slim image containing only the release binaries.
ARG RUST_TOOLCHAIN=1.85.0
ARG LLVM_VERSION=10

# -----------------------------------------------------------------------------
# Build stage
# -----------------------------------------------------------------------------
FROM ubuntu:20.04 AS builder
ARG RUST_TOOLCHAIN
ARG LLVM_VERSION

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_NET_OFFLINE=false \
    RUSTUP_TOOLCHAIN=${RUST_TOOLCHAIN} \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    LLVM_SYS_100_PREFIX=/usr/lib/llvm-${LLVM_VERSION} \
    LLVM_VERSION=${LLVM_VERSION}

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    clang-${LLVM_VERSION} \
    libllvm${LLVM_VERSION} \
    llvm-${LLVM_VERSION} \
    llvm-${LLVM_VERSION}-dev \
    llvm-${LLVM_VERSION}-runtime \
    lld-${LLVM_VERSION} \
    cmake \
    make \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain ${RUST_TOOLCHAIN} --profile default

WORKDIR /app

# Dependency layer: cache registry across builds.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY library/ library/

# Dependency layer: cache registry across builds.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo fetch

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --workspace && \
    install -d /usr/local/bin && \
    install -m 0755 \
        target/release/gala \
        target/release/gala-fmt \
        target/release/gala-lsp \
        target/release/gala-lint \
        target/release/gala-pkg \
        /usr/local/bin/

# -----------------------------------------------------------------------------
# Dev stage (dev container / CI parity): formatting, linting, auditing
# -----------------------------------------------------------------------------
FROM builder AS dev

RUN rustup component add rustfmt clippy && \
    cargo install cargo-audit cargo-deny --locked

WORKDIR /app
VOLUME ["/app", "/usr/local/cargo/registry"]

# -----------------------------------------------------------------------------
# Runtime stage
# -----------------------------------------------------------------------------
FROM ubuntu:20.04 AS runtime
ARG LLVM_VERSION

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libgcc-s1 \
    libllvm${LLVM_VERSION} \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/gala \
    /usr/local/bin/gala-fmt \
    /usr/local/bin/gala-lsp \
    /usr/local/bin/gala-lint \
    /usr/local/bin/gala-pkg \
    /usr/local/bin/

WORKDIR /workspace
ENTRYPOINT ["gala"]
CMD ["--help"]
