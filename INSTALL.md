# Installing Gala

## Prerequisites

- **Rust toolchain** 1.75 or later (install via [rustup](https://rustup.rs/))
- **Git** (for cloning the repository)

## Build from Source

```bash
# Clone the repository
git clone https://github.com/your-org/gala
cd gala

# Build the compiler and tools
cargo build --release --workspace

# Install binaries to ~/.cargo/bin
cp target/release/gala ~/.cargo/bin/
cp target/release/gala-fmt ~/.cargo/bin/
cp target/release/gala-lsp ~/.cargo/bin/

# Verify installation
gala --version
```

## Verify

```bash
echo 'fn main() -> Int { print("Hello!"); return 0; }' | gala -
```

## IDE Support

Install the Gala LSP and configure your editor:

- **VS Code**: Install `gala-vscode` from the marketplace
- **Neovim**: Add `gala-lsp` to your LSP config
- **Helix**: Add to `languages.toml` (see `docs/editor-support.md`)
