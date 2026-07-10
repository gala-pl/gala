# Installing Gala

## Prerequisites

- **Rust toolchain** 1.75 or later (install via [rustup](https://rustup.rs/))
- **Git** (for cloning the repository)
- **LLVM 17+** (for QIR/classical backends, optional)

## Quick Start (macOS with Homebrew)

```bash
# Install all dependencies
brew bundle --file=.homebrew/Brewfile

# Set up environment
source .homebrew/configure.sh

# Build the compiler and tools
cargo build --release --workspace

# Install binaries to ~/.cargo/bin
cp target/release/gala ~/.cargo/bin/
cp target/release/gala-fmt ~/.cargo/bin/
cp target/release/gala-lsp ~/.cargo/bin/

# Verify installation
gala --version
```

## Build without LLVM (frontend-only)

```bash
cargo build --release --workspace
```

This builds the lexer, parser, type checker, HIR, formatter, and package
manager — everything except the QIR/LLVM backends and quantum simulator.

## Build with All Backends

```bash
# Install LLVM for QIR emission
brew install llvm@17

# Set up environment
source .homebrew/configure.sh

# Build everything
cargo build --release --workspace
```

## Verify

```bash
echo 'fn main() -> Int { print("Hello!"); return 0; }' > hello.gala
gala build hello.gala
```

## IDE Support

Install the Gala LSP and configure your editor:

- **VS Code**: Install `gala-vscode` from the marketplace
- **Neovim**: Add `gala-lsp` to your LSP config
- **Helix**: Add to `languages.toml`

## Homebrew Details

See `.homebrew/README.md` for detailed Homebrew configuration, including:
- `Brewfile` — all external dependencies
- `gala.rb` — Homebrew formula for `brew install gala`
- `configure.sh` — environment setup script
