# gala-fmt

Canonical code formatter for the Gala programming language. Ensures consistent code style across all Gala projects.

## Overview

`gala-fmt` reads a `.gala` source file and outputs a formatted version with consistent indentation and whitespace. It is designed to integrate with editor save hooks, pre-commit checks, and CI pipelines.

## Formatting Rules

- **Indentation:** 4 spaces per level
- **Braces:** Opening `{` increases indent, closing `}` decreases indent (matching Rust convention)
- **Parentheses:** Opening `(` increases indent, closing `)` decreases indent
- **Blank lines:** Preserved as-is
- **Leading/trailing whitespace:** Stripped from each line
- **Line content:** Trimmed and re-indented (no trailing whitespace)

## Usage

```bash
# Build
cargo build --workspace

# Format a file
cargo run --bin gala-fmt -- examples/hello_world.gala

# Format and write back
cargo run --bin gala-fmt -- examples/hello_world.gala > formatted.gala
# or with a wrapper script for in-place formatting
```

## Planned Enhancements

- [ ] **In-place formatting** (`--in-place` / `-i` flag to overwrite the source file)
- [ ] **Check mode** (`--check` flag, exit non-zero if formatting differs — useful for CI)
- [ ] **Range formatting** — format only a selected range (for LSP integration)
- [ ] **Configurable indent width** (`--indent-width` flag)
- [ ] **AST-based formatting** — use the compiler's parsed AST for semantically-aware formatting rather than brace counting
- [ ] **LSP integration** — plug into `gala-lsp` for format-on-save

## Dependencies

- `gala-compiler` — uses the compiler's lexer and parser for AST-based formatting

## Integration

### VS Code

Configure format-on-save in `.vscode/settings.json`:

```json
{
  "[gala]": {
    "editor.defaultFormatter": "gala.gala-fmt",
    "editor.formatOnSave": true
  }
}
```

### Pre-commit

Add to `.pre-commit-config.yaml`:

```yaml
- repo: local
  hooks:
    - id: gala-fmt
      name: gala-fmt
      entry: cargo run --bin gala-fmt -- --check
      language: system
      files: '\.gala$'
```
