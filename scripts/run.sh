#!/usr/bin/env bash
set -euo pipefail

GALA_BIN="cargo run --bin gala --"

case "${1:-}" in
    build)
        cargo build --workspace
        ;;
    test)
        cargo test --workspace
        ;;
    lint)
        cargo clippy --workspace -- -D warnings
        ;;
    fmt)
        cargo fmt --all
        ;;
    fmt-check)
        cargo fmt --check
        ;;
    clean)
        cargo clean
        ;;
    doc)
        cargo doc --workspace --no-deps --open
        ;;
    run)
        shift
        $GALA_BIN "$@"
        ;;
    fmt-gala)
        shift
        cargo run --bin gala-fmt -- "$@"
        ;;
    docs-dev)
        cd apps/docs && bun dev
        ;;
    docs-build)
        cd apps/docs && bun next build
        ;;
    precommit)
        pre-commit run --all-files
        ;;
    *)
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  build         Build all crates"
        echo "  test          Run all tests"
        echo "  lint          Run clippy"
        echo "  fmt           Format all Rust code"
        echo "  fmt-check     Check formatting"
        echo "  clean         Clean build artifacts"
        echo "  doc           Build and open docs"
        echo "  run <file>    Compile and run a .gala file"
        echo "  fmt-gala <f>  Format a .gala file"
        echo "  docs-dev      Start docs dev server"
        echo "  docs-build    Build docs site"
        echo "  precommit     Run pre-commit hooks"
        ;;
esac
