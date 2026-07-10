#!/usr/bin/env bash
set -euo pipefail

echo "=== Gala Test Coverage ==="
echo ""

# Build test binaries with instrumentation
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/prof/gala-%p-%m.profraw"

# Run tests
cargo test --workspace

# Generate coverage report
if command -v grcov &> /dev/null; then
    grcov target/prof/ \
        --source-dir . \
        --output-type html \
        --binary-path target/debug/ \
        --output-dir target/coverage/
    echo "Coverage report: target/coverage/index.html"
elif command -v cargo-llvm-cov &> /dev/null; then
    cargo llvm-cov --workspace --html
    echo "Coverage report: target/llvm-cov/html/index.html"
else
    echo "Install grcov or cargo-llvm-cov for coverage reports"
    echo "  cargo install grcov"
    echo "  cargo install cargo-llvm-cov"
fi

# Clean up profile data
rm -rf target/prof/
