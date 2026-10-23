# gala-mcp

Model Context Protocol server for the Gala language. Speaks JSON-RPC 2.0 over stdio
(newline-delimited). Written in TypeScript for Bun with no third-party dependencies.

## Run

```sh
cd tools/gala-mcp
bun src/index.ts                       # serves MCP on stdin/stdout
GALA_BIN=/path/to/gala bun src/index.ts  # override the binary
```

The default binary is `<repo>/target/debug/gala`. Build it with `cargo build --bin gala`.

## Tools

| Tool | Status |
| --- | --- |
| `gala_check` | Implemented. Returns `gala.check.v1` JSON, validated against `diagnostic.v1`. |
| `gala_build` | Implemented for text IR. `emit_gir: true` returns an error result (no GIR JSON in the CLI). |
| `gala_run` | Implemented. Returns prints, the result line, and any histogram. |
| `gala_test` | Runs `#[test]` functions. Property checks return an error result (no property runner). |
| `gala_explain` | Implemented via `gala explain --json`. |
| `gala_fix` | Not implemented. The CLI has no `fix` subcommand; returns an error result. |
| `gala_gir` | Not implemented. The CLI cannot emit `gir.v1` JSON; returns an error result. |
| `gala_lsp_query` | Not available. This server does not connect to `gala-lsp`; returns an error result. |

## Resources

- `gala://diagnostics/{file}`: check output for an absolute path to a `.gala` file (percent-encoded)
- `gala://gir/{file}`: not available
- `gala://schema/diagnostic.v1`, `gala://schema/gir.v1`, `gala://schema/property_result.v1`

## Prompts

- `gala/debug-error` (`file`): diagnostics plus source, with a request to explain them
- `gala/write-quantum-fn` (`task`, `constraints?`)
- `gala/optimize-circuit` (`file`, `objectives?`): source plus text IR

## Test

```sh
cd tools/gala-mcp && bun test
```

The tests start the server over stdio and cover `initialize`, `tools/list`, `gala_check` (E0412),
`gala_run` (bell_run prints `true`), error results for unavailable tools, resources, and prompts.
