#!/usr/bin/env bash
set -euo pipefail

# Gala Bootstrap Script
# Installs build dependencies and sets up the development environment.

echo "=== Gala Bootstrap ==="

# Check for Rust toolchain
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Rust version: $(rustc --version)"

# Install pinned toolchain
if [ -f rust-toolchain.toml ]; then
    echo "Installing pinned toolchain..."
    rustup show
fi

# Install pre-commit
if ! command -v pre-commit &> /dev/null; then
    echo "Installing pre-commit..."
    pip install pre-commit
fi

# Set up pre-commit hooks
if [ -f .pre-commit-config.yaml ]; then
    echo "Installing pre-commit hooks..."
    pre-commit install
fi

# Build the workspace
echo "Building workspace..."
cargo build --workspace

# Run tests
echo "Running tests..."
cargo test --workspace

echo ""
echo "=== Bootstrap complete ==="
echo "Run 'cargo run --bin gala -- --input examples/hello_world.gala' to get started."
