# Build stage
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY compiler/ compiler/
COPY library/ library/
COPY tools/ tools/

RUN cargo build --release --workspace

# Runtime stage
FROM alpine:3.20

RUN apk add --no-cache libgcc

COPY --from=builder /app/target/release/gala /usr/local/bin/gala
COPY --from=builder /app/target/release/gala-fmt /usr/local/bin/gala-fmt
COPY --from=builder /app/target/release/gala-lsp /usr/local/bin/gala-lsp
COPY --from=builder /app/target/release/gala-cli /usr/local/bin/gala-cli
COPY --from=builder /app/target/release/gala-lint /usr/local/bin/gala-lint
COPY --from=builder /app/target/release/gala-pkg /usr/local/bin/gala-pkg

WORKDIR /workspace
ENTRYPOINT ["gala"]
CMD ["--help"]
