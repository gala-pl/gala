# Gala Homebrew Tap

**Official Homebrew tap for the [Gala programming language](https://github.com/gala-lang/gala).**

## Quick Install

```bash
# Add the tap
brew tap gala-lang/gala

# Install Gala (frontend-only, no LLVM)
brew install gala

# Install Gala with all backends
brew install gala --with-llvm --with-sim
```

## Available Formulae

| Formula | Description |
|---------|-------------|
| `gala` | The Gala compiler and toolchain |

## Build Options

| Option | Description |
|--------|-------------|
| `--with-llvm` | Enable QIR emission and classical codegen via LLVM |
| `--with-sim` | Enable the built-in quantum simulator |
| `--with-tree-sitter` | Build tree-sitter grammar for syntax highlighting |

## Dependencies

- **Rust** — installed automatically by the formula
- **LLVM 17** — optional, for `--with-llvm`
- **CMake** — installed automatically
- **pkg-config** — installed automatically

## Building from Source (without Homebrew)

```bash
# Clone and build
git clone https://github.com/gala-lang/gala.git
cd gala
source homebrew/configure.sh
cargo build --workspace
```

See the project `INSTALL.md` for detailed build instructions.
