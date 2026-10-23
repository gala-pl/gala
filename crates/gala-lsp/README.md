# gala-lsp

Language server for Gala. It speaks JSON-RPC 2.0 over stdio with `Content-Length` framing.

```
cargo run -p gala-lsp          # reads LSP on stdin, writes on stdout
```

## Standard features

| Request or notification | Behavior |
|---|---|
| `initialize`, `initialized`, `shutdown`, `exit` | Lifecycle. Advertises full text sync, hover, definition, completion. |
| `textDocument/didOpen`, `didChange`, `didClose` | Full-document sync. Every change recompiles and publishes `textDocument/publishDiagnostics`. Syntax errors are published even when the file does not compile. |
| `textDocument/hover` | Function signature with its effect (`fn f(x: Float) -> Measured<Bool> prob`), local type, or expression type. `null` otherwise. |
| `textDocument/definition` | Location of the name of a function or local in the document. `null` if unresolved. |
| `textDocument/completion` | Builtins (`qubit`, `measure`, `h`, `cx`, `print`, ...) and top-level functions of the document. Kind is Function. |

Diagnostics use severity 1 for errors and 2 for warnings, the E-code as `code`, and the
message as `message`. Positions count UTF-16 code units, as LSP requires.

When a file compiles with errors (for example E0412), hover and definition still work on the
parsed HIR and types. Symbols from imported library modules are not indexed for hover or
definition.

## Custom methods (`gala/*`)

All take `{ "textDocument": { "uri": ... }, "position": { "line": n, "character": n } }`
unless noted. Each returns `null` when there is no answer.

- `gala/typeAtPosition` returns `{ "type": "Int" }`. For a function name it returns the
  function type, for example `fn(Float) -> Measured<Bool>`.
- `gala/effectAtPosition` returns `{ "effect": "pure" | "quantum" | "prob" }` for the function
  enclosing the position.
- `gala/girAtPosition` returns `{ "function": "name", "gir": "..." }`, where `gir` is that
  function's section of the GIR dump. It is `null` when the program does not compile to GIR.
- `gala/suggestFixes` takes only `textDocument` and returns an array of
  `{ "code", "title", "edits": [{ "range", "newText" }] }` built from every diagnostic's
  suggested fixes. It returns `[]` when no diagnostic carries a fix. Few diagnostics currently
  carry fixes, so the array is often empty.

## Not implemented

- `tower-lsp` is not used. It is not in the offline cargo cache, so the protocol is handled
  directly.
- No incremental sync, no references, rename, or semantic tokens.
