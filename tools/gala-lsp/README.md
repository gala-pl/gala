# gala-lsp

Language Server Protocol (LSP) implementation for the Gala programming language. Provides IDE features — diagnostics, completions, hover information, go-to-definition — by sharing the `salsa` query graph with the compiler.

## Status

**Early scaffold.** The server currently accepts JSON-RPC messages over stdin/stdout and returns a no-op diagnostics notification. Full LSP features are under development.

## Architecture

```
Editor (VS Code / Vim / Helix / Zed)
    │  LSP (JSON-RPC over stdin/stdout)
    ▼
gala-lsp
    │  shares salsa query graph
    ▼
gala-compiler (library)
```

The LSP server runs as a subprocess launched by the editor. It communicates via the Language Server Protocol over stdin/stdout, sharing the compiler's incremental computation graph to provide low-latency responses as the user edits.

## Planned Features

- [ ] **Live diagnostics** — errors and warnings as you type, leveraging the compiler's incremental `salsa` query graph
- [ ] **Hover information** — type signatures, documentation, and quantum effect annotations
- [ ] **Go-to-definition** — navigate to symbol declarations across files
- [ ] **Completions** — context-aware suggestions for keywords, types, functions, and gates
- [ ] **Code actions** — quick fixes for common errors, auto-import, add missing returns
- [ ] **Inline circuit diagrams** — visualize quantum circuits inline (VS Code webview)
- [ ] **Signature help** — parameter hints for function calls
- [ ] **Document symbols** — outline view of modules, functions, types
- [ ] **Workspace symbols** — search across the entire project
- [ ] **Formatting** — integrate with `gala-fmt`

## Usage

```bash
# Build
cargo build --workspace

# Run (editors launch this automatically)
gala-lsp
```

The server reads JSON-RPC messages from stdin and writes responses to stdout. Editors are configured to launch it automatically when opening `.gala` files.

## Dependencies

- `gala-compiler` — shared compiler library for parsing, type checking, and diagnostics
- `serde` / `serde_json` — JSON-RPC message serialization

## Editor Integration

The LSP is configured in each editor extension:

| Editor | Configuration |
|--------|---------------|
| **VS Code** | `extensions/vscode/` — built-in LSP client |
| **Vim/Neovim** | Configure via `vim.lsp.start()` or coc.nvim |
| **Helix** | `extensions/helix/languages.toml` |
| **Zed** | `extensions/zed/` — LSP adapter |
| **IntelliJ** | `extensions/intellij/` — LSP-based plugin |
