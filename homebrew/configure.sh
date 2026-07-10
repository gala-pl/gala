#!/usr/bin/env bash
# Gala Build Environment Configuration
# Usage: source homebrew/configure.sh

set -euo pipefail

GALA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "==> Gala build environment"
echo "    root: $GALA_ROOT"

# Try to locate LLVM (required for QIR/classical backends)
for llvm_ver in 17 18 19 16 15; do
  prefix="$(brew --prefix "llvm@${llvm_ver}" 2>/dev/null || true)"
  if [ -n "$prefix" ] && [ -d "$prefix" ]; then
    export LLVM_SYS_100_PREFIX="$prefix"
    export PATH="$prefix/bin:$PATH"
    echo "    LLVM: $prefix (version $llvm_ver)"
    break
  fi
done

if [ -z "${LLVM_SYS_100_PREFIX:-}" ]; then
  if command -v llvm-config &>/dev/null; then
    export LLVM_SYS_100_PREFIX="$(llvm-config --prefix)"
    echo "    LLVM: $LLVM_SYS_100_PREFIX (from PATH)"
  else
    echo "    LLVM: not found — QIR/classical backends disabled"
    echo "           Install: brew install llvm@17"
  fi
fi

# Check Rust
if command -v rustc &>/dev/null; then
  echo "    Rust:  $(rustc --version)"
else
  echo "    Rust:  not found — install via https://rustup.rs"
fi

echo ""
echo "==> Build: cargo build --workspace"
