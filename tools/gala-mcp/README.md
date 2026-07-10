# Gala MCP Server

Model Context Protocol server for the Gala quantum-classical programming language. Exposes Gala's compiler, simulator, and analysis tools to AI agents.

## Installation

```bash
# From the gala repo root
cd tools/gala-mcp
npm install
npm run build
```

## Usage

### With Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "gala": {
      "command": "npx",
      "args": ["gala-mcp"],
      "cwd": "/path/to/gala/tools/gala-mcp"
    }
  }
}
```

### With VS Code (via MCP extension)

```json
{
  "mcp": {
    "servers": {
      "gala": {
        "command": "npx",
        "args": ["gala-mcp"],
        "cwd": "/path/to/gala/tools/gala-mcp"
      }
    }
  }
}
```

### Direct stdio

```bash
npx gala-mcp
```

## Tools

| Tool | Description | Input | Output |
|------|-------------|-------|--------|
| `gala_check` | Type-check + linearity + effects | `{source, file_path?}` | `{diagnostics, gir?}` |
| `gala_build` | Build with optional GIR/QIR emission | `{source, emit_gir?, gir_version?, target?}` | `{gir}` |
| `gala_run` | Execute on simulator | `{source, shots?, seed?}` | `{results, circuit?}` |
| `gala_test` | Run quantum property tests | `{source, properties[]}` | `{passed, results[]}` |
| `gala_fix` | Apply suggested fixes | `{source}` | `{applied[], remaining[]}` |
| `gala_explain` | Get error explanation | `{code: "E0412"}` | `{markdown, examples[]}` |
| `gala_gir` | Get GIR for semantic analysis | `{source}` | `{gir, version}` |
| `gala_lsp_query` | Query LSP (hover, completion, GIR at position) | `{source, file_path, position, query_type}` | `{hover?, completion?, ...}` |

## Resources

| URI | Description |
|-----|-------------|
| `gala://schema/diagnostic.v1` | JSON schema for diagnostics |
| `gala://schema/gir.v1` | JSON schema for GIR |
| `gala://schema/property_result.v1` | JSON schema for property test results |

## Prompts

| Prompt | Description |
|--------|-------------|
| `gala/debug-error` | Explain a diagnostic's quantum physics and suggest fixes |
| `gala/write-quantum-fn` | Generate Gala code for a quantum algorithm |
| `gala/optimize-circuit` | Optimize a circuit for a specific backend |

## Example Usage

### Type-check code
```json
{
  "tool": "gala_check",
  "arguments": {
    "source": "fn bell() -> Qubits<2> quantum { let q = qalloc(2); h(q[0]); cx(q[0], q[1]); q }"
  }
}
```

### Run on simulator
```json
{
  "tool": "gala_run",
  "arguments": {
    "source": "fn main() -> Measured<Bool> prob { let q = qalloc(1); h(q[0]); measure(q[0]) }",
    "shots": 1024
  }
}
```

### Verify unitarity
```json
{
  "tool": "gala_test",
  "arguments": {
    "source": "#[property(unitary)] fn qft4(q: Qubits<4>) -> Qubits<4> quantum { ... }",
    "properties": ["unitary", "reversible"]
  }
}
```

### Get GIR for analysis
```json
{
  "tool": "gala_gir",
  "arguments": {
    "source": "fn bell() -> Qubits<2> quantum { ... }"
  }
}
```

## Prerequisites

- Gala compiler binary (`gala`) in PATH or at `../../target/release/gala`
- Build with: `cargo build --release --workspace` from repo root

## Schema Versioning

All JSON schemas are versioned (`v1`, `v2`, etc.) and published at:
- `https://gala-lang.org/schemas/diagnostic.v1.json`
- `https://gala-lang.org/schemas/gir.v1.json`
- `https://gala-lang.org/schemas/property_result.v1.json`

Agents should pin to a specific schema version for stability.

## Development

```bash
# Watch mode
npm run dev

# Run tests
npm test

# Inspect with MCP Inspector
npm run inspect
```