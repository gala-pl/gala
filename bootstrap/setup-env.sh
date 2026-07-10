#!/usr/bin/env bash
# Source this file to set up environment variables for Gala development.

export CARGO_TERM_COLOR=always
export RUSTFLAGS="-D warnings"
export RUST_BACKTRACE=1

# Add local cargo bin to PATH
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "Gala development environment loaded"
echo "  CARGO_TERM_COLOR=$CARGO_TERM_COLOR"
echo "  RUSTFLAGS=$RUSTFLAGS"
echo "  RUST_BACKTRACE=$RUST_BACKTRACE"
