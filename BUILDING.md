# Building Gala

Gala is a hybrid quantum-classical compiler targeting Rust edition 2021 with
the pinned toolchain `stable-2026-04-14` (see `rust-toolchain.toml`). The
`inkwell` codegen backend requires **LLVM 10** (`llvm10-0` feature).

The `extensions/tree-sitter-gala` crate is intentionally excluded from the
workspace (commented out in `Cargo.toml`) and is built separately.

## 1. Cargo (recommended for development)

```bash
# Pin the toolchain locally (rustup will install it on demand)
rustup toolchain install stable-2026-04-14

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
cargo run --bin gala -- --help
```

`Cargo.lock` is committed — builds are reproducible given the pinned toolchain.

## 2. Docker

Multi-stage image with the pinned toolchain, LLVM 10 dev libraries, and all
release binaries (`gala`, `gala-fmt`, `gala-lsp`, `gala-cli`, `gala-lint`,
`gala-pkg`).

```bash
# Build
docker build -t gala:latest .

# Run
docker run --rm gala:latest --help

# Dev container (mounts source, caches cargo)
docker compose run gala-dev cargo clippy --workspace --all-targets -- -D warnings

# CI parity (fmt, clippy, build, test in one container)
docker compose run gala-ci
```

`docker-compose.yml` defines `gala-dev` (dev target), `gala-ci` (CI parity),
and `gala-release` (stages release binaries into `./dist`).

## 3. Bazel

Workspace with `rules_rust` 0.50 and `crate_universe` pinned from
`Cargo.lock` (`Cargo.Bazel.lock`). Third-party crates come from `@crates`;
LLVM 10 is provided by the `llvm` repository (system prefix, overridable via
`LLVM_PREFIX`).

```bash
bazel build //...
bazel test //...

# Enable the LLVM 10 inkwell backend (classical codegen):
bazel build //crates/gala-codegen-classical --features=classical \
  --action_env=LLVM_SYS_100_PREFIX=/usr
```

Regenerate the Bazel lockfile after `Cargo.lock` changes:

```bash
REPIN=1 bazel sync --only=crates
```

Bazel builds default features (no inkwell) so no LLVM toolchain is required
unless `classical` is enabled.

## 4. Nix

Flake with a dev shell, packages, and checks:

```bash
# Development shell (rustc, cargo, rustfmt, clippy, clang, LLVM 10,
# cargo-audit, cargo-deny)
nix develop

# Build all release binaries
nix build .#gala

# CI checks: build, test, clippy, fmt
nix flake check
```

The flake pins `rust-overlay` (toolchain `stable-2026-04-14`) and
`llvmPackages_10`. Recompute `cargoHash` in `flake.nix` after `Cargo.lock`
changes by running `nix build .#gala` and copying the hash from the error
message.

## 5. CI

GitHub Actions matrix:

- `ci.yml` — cargo build/test/clippy/fmt on ubuntu + macOS
- `ci.yml` — docker build + smoke test
- `ci.yml` — bazel build + test (LLVM 10 installed on the runner)
- `ci.yml` — nix flake check + build

## 6. Release

Triggered by pushing a `v*.*.*` tag (`release.yml`):

- Release binaries for ubuntu-latest and macos-latest, uploaded to the GitHub
  release
- Docker image built and pushed to `ghcr.io/${{ github.repository }}`
- Nix flake: bump `cargoHash` for the new `Cargo.lock` and open a PR manually
