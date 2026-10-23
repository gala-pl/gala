---
name: gala-pkg
description: Work with Gala's package manager — gala.toml, resolver, registry client, gala add/remove/update, lockfile
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's package manager (`crates/gala-pkg`) handles **package manifests** (`gala.toml`), **dependency resolution**, **registry client**, and **lockfile** (`gala.lock`). Commands: `gala add`, `gala remove`, `gala update`, `gala pkg publish`.

> **Note on file layout:** The crate currently lives in `crates/gala-pkg/src/lib.rs` + `crates/gala-pkg/src/main.rs`. As it matures it may be split into `manifest.rs`, `resolver.rs`, `registry.rs`, `lockfile.rs`, `commands.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-pkg/src/lib.rs` — Package management logic
- `crates/gala-pkg/src/main.rs` — CLI entry point

## Common Tasks

### Modify manifest format
1. Edit `lib.rs` — `GalaToml` struct, TOML schema
2. Add new fields (e.g., `backend`, `target`, `noise-model`)
3. Update validation rules
4. Migration: auto-upgrade old manifests

### Improve resolver
1. Edit `lib.rs` — semver constraint solving
2. Handle git/path dependencies
3. Support workspace inheritance
4. Test: complex dependency graphs, conflict resolution

### Add registry feature
1. Edit `lib.rs` — API endpoints, authentication
2. `publish`: sign package, upload to registry
3. `fetch`: download, verify checksum, cache
4. `search`: query registry index

### Lockfile improvements
1. Edit `lib.rs` — deterministic generation
2. Include content hashes for reproducibility
3. `gala pkg verify` — check lockfile matches manifest
4. JSON output: `gala pkg add --json`

## Testing

```bash
cargo test -p gala-pkg                      # Unit tests
cargo test --workspace pkg                  # All pkg tests
cargo run --bin gala -- pkg new mypkg       # Create package
cargo run --bin gala -- add gala.vqa        # Add dependency
cargo run --bin gala -- pkg publish --dry-run  # Test publish
```

## Conventions

- **`gala.toml` at package root** — mirrors Cargo.toml conventions
- **Semver** — `^1.2.3`, `>=1.0,<2.0`, git refs, paths
- **`gala.lock` committed** — reproducible builds
- **Registry optional** — local paths work without registry
- **JSON output for agents** — `--json` flag on all commands
- No `unwrap()` on user paths

## Manifest Schema

```toml
[package]
name = "my_pkg"
version = "0.1.0"
edition = "2024"
description = "..."
authors = ["..."]
license = "Apache-2.0"
repository = "..."

[dependencies]
gala.std = "0.1"
gala.vqa = { git = "https://github.com/...", rev = "abc123" }
local_pkg = { path = "../local" }

[backend]
# Optional: default backend for `gala run`
default = "simulator"

[profile.release]
opt-level = 3
```

## Debugging

```bash
# Create new package
cargo run --bin gala -- pkg new my_pkg

# Add dependency
cargo run --bin gala -- add gala.vqa
cargo run --bin gala -- add gala.vqa --json

# Resolve and show tree
cargo run --bin gala -- pkg tree

# Verify lockfile
cargo run --bin gala -- pkg verify

# Publish (dry run)
cargo run --bin gala -- pkg publish --dry-run
```