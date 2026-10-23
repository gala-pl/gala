---
name: gala-lsp
description: Work with Gala's Language Server — tower-lsp, salsa sharing, diagnostics, hover, go-to-def, completion, circuit diagrams
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's LSP (`crates/gala-lsp`) is built on **tower-lsp** and shares the **salsa query database** with the compiler. Provides live diagnostics, hover (types+effects), go-to-definition, rename, completion, and **inline circuit diagrams**.

Key files:
- `crates/gala-lsp/src/server.rs` — LSP server, request handlers
- `crates/gala-lsp/src/diagnostics.rs` — Push diagnostics on change
- `crates/gala-lsp/src/hover.rs` — Type/effect hover, circuit diagram
- `crates/gala-lsp/src/completion.rs` — Completion items (keywords, stdlib, snippets)
- `crates/gala-lsp/src/goto.rs` — Go-to-definition, references
- `crates/gala-lsp/src/rename.rs` — Rename with salsa invalidation
- `crates/gala-lsp/src/custom.rs` — Custom methods for agents (`gala/girAtPosition`, etc.)

## Common Tasks

### Add LSP feature
1. Implement handler in appropriate module (`hover.rs`, `completion.rs`, etc.)
2. Register in `server.rs` capabilities
3. Use salsa queries: `db.hir(file)`, `db.type_of(def_id)`, `db.gir(def_id)`
4. For circuit diagrams: query `db.gir(def_id)`, render ASCII/UTF-8 circuit
5. Test in VS Code with `extensions/vscode/`

### Add custom method for agents
1. Edit `custom.rs` — define method name (`gala/girAtPosition`)
2. Implement handler returning typed JSON (GIR schema v1)
3. Register in `server.rs` with `ServerCapabilities::experimental`
4. Test with MCP Inspector or VS Code extension

### Improve diagnostics latency
1. Diagnostics pushed via `textDocument/publishDiagnostics`
2. Use salsa incremental: only re-check changed files
3. Debounce: batch rapid edits (configurable)
4. Profile: `RUST_LOG=gala_lsp=debug`

### Fix go-to-definition / rename
1. Uses `db.resolve(def_id)` and `db.module_graph(crate)`
2. For rename: compute all references via salsa, apply edits
3. Test cross-file, cross-crate

## Testing

```bash
cargo test -p gala-lsp                      # Unit tests
cargo test --workspace lsp                  # All LSP tests
# Integration test: start LSP, send requests via JSON-RPC
cargo run --bin gala-lsp -- --stdio  # Manual test
```

## Conventions

- **Shares salsa DB** with compiler — same queries, incremental
- **Error recovery** — works on broken files (parser produces partial AST)
- **Latency budget** — <100ms for typical requests
- **Circuit diagrams** — inline UTF-8 rendering of quantum circuit
- **Agent-friendly** — custom methods expose semantic info as JSON
- No `unwrap()` on user paths

## Key Custom Methods (Agent SDK)

```json
// gala/girAtPosition
{ "position": { "line": 10, "character": 5 } }
→ { "gir": { ... }, "schema": "gir.v1.json" }

// gala/typeAtPosition
{ "position": { "line": 10, "character": 5 } }
→ { "type": "Qubits<4>", "effect": "quantum" }

// gala/effectAtPosition
{ "position": { "line": 10, "character": 5 } }
→ { "effect": "quantum" }

// gala/suggestFixes
{ "position": { "line": 10, "character": 5 }, "diagnostic_code": "E0412" }
→ { "fixes": [{ "title": "Remove use", "edits": [...] }] }
```

## Debugging

```bash
# Start LSP server manually
cargo run --bin gala-lsp -- --stdio

# Test with VS Code (install extensions/vscode/)
code --install-extension extensions/vscode/

# Debug LSP messages
RUST_LOG=gala_lsp=debug,tower_lsp=debug cargo run --bin gala-lsp -- --stdio

# Test custom method
echo '{"jsonrpc":"2.0","id":1,"method":"gala/girAtPosition","params":{"position":{"line":5,"character":10}}}' | cargo run --bin gala-lsp -- --stdio
```