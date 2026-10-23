---
name: gala-cli
description: Work with Gala's CLI — clap commands, build/check/run/test/fmt/repl/add/pkg, JSON output modes
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's CLI (`crates/gala-cli`) is the **user-facing `gala` binary** built with **clap**. Commands: `build`, `check`, `run`, `test`, `fmt`, `repl`, `add`, `remove`, `update`, `pkg`, `explain`, `fix`, `new`, `init`.

> **Note on file layout:** The crate currently lives in `crates/gala-cli/src/main.rs`. As it matures it may be split into `commands/`, `output.rs`, `config.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `main.rs`.

Key files:
- `crates/gala-cli/src/main.rs` — Entry point, command dispatch

## Common Tasks

### Add new CLI command
1. Create function in `main.rs` (or new file if split)
2. Define clap `Args` struct with `#[command]` attributes
3. Implement `run` function using `gala-driver` / other crates
4. Register in command enum in `main.rs`
5. Add `--json` output support
6. Add help text, examples

### Add `--json` to existing command
1. Edit command's `Args` — add `json: bool` flag
2. In `run`, collect structured result
3. Use output formatting
4. Ensure schema versioned (e.g., `build.v1.json`)

### Modify build pipeline flags
1. Edit `main.rs` — `--emit` variants (ast, hir, gir, gir-opt, qir, llvm, clif)
2. Add new emission target in `gala-driver`
3. Test: `gala build --emit gir examples/hello.gala`

### Add REPL feature
1. Edit `main.rs` — incremental evaluation loop
2. Use `gala-driver` for incremental compilation
3. Add visualization: state vector, Bloch sphere, histogram
4. `--json` mode for programmatic use

## Testing

```bash
cargo test -p gala-cli                      # Unit tests
cargo test --workspace cli                  # All CLI tests
cargo run --bin gala -- --help              # Help
cargo run --bin gala -- build examples/hello.gala --emit gir
cargo run --bin gala -- check examples/broken.gala --json
cargo run --bin gala -- run examples/bell.gala --shots 1000
cargo run --bin gala -- repl --json
```

## Conventions

- **All commands support `--json`** — structured output for agents
- **Exit codes**: 0 = success, 1 = user error (diagnostics), 2 = internal error
- **`gala check --json --fail-on-warnings`** — single command for agent CI
- **Conventional help** — `gala <cmd> --help` shows usage, examples
- **Config via `gala.toml`** — project-level settings
- No `unwrap()` on user paths

## Command Reference

| Command | Purpose | Key Flags |
|---------|---------|-----------|
| `gala build` | Compile to artifact | `--emit [gir|qir|llvm|clif]`, `--target` |
| `gala check` | Type-check only | `--json`, `--fail-on-warnings` |
| `gala run` | Execute on backend | `--backend`, `--shots`, `--seed`, `--noise` |
| `gala test` | Run property/conformance | `--property`, `--json`, `--generate-corpus` |
| `gala fmt` | Format code | `--check`, `--diff`, `--stdin`, `--width` |
| `gala repl` | Interactive session | `--json`, `--visualize` |
| `gala explain` | Diagnostic docs | `<code>`, `--markdown`, `--json` |
| `gala fix` | Apply suggested fixes | `--apply`, `--dry-run` |
| `gala new` | Create package/project | `--bin`, `--lib` |
| `gala add` | Add dependency | `--json` |
| `gala pkg` | Package management | `publish`, `verify`, `tree` |

## Debugging

```bash
# Verbose CLI
RUST_LOG=gala_cli=debug cargo run --bin gala -- build examples/hello.gala

# Test JSON output
cargo run --bin gala -- check examples/broken.gala --json | jq

# Profile CLI startup
cargo build --release --bin gala
time ./target/release/gala --version
```