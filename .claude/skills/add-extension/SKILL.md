---
name: add-extension
description: Add or update Gala syntax highlighting / editor extension for a given editor
disable-model-invocation: true
allowed-tools: Bash(cd *) Bash(cargo *) Bash(cargo run --bin gala *) Read Write Grep Glob Edit
---

## Task

$ARGUMENTS

## Instructions

Add or update a Gala editor extension following the existing patterns.

### Extension Locations

- VS Code: `extensions/vscode/`
- IntelliJ: `extensions/intellij/`
- Vim: `extensions/vim/`
- Helix: `extensions/helix/`
- Zed: `extensions/zed/`
- Sublime Text: `extensions/sublime/`
- TextMate: `extensions/textmate/`
- highlight.js: `extensions/highlight-js/`
- Prism.js: `extensions/prism/`
- tree-sitter: `extensions/tree-sitter-gala/`
- bat: `extensions/bat/`
- cursor: `extensions/cursor/`

### Requirements

1. Study an existing extension (e.g., Vim or Helix) to understand the keyword set and syntax patterns
2. Check `examples/` for valid `.gala` sample programs to test against
3. Implement the syntax definitions following the target editor's conventions
4. Test with `python -m unittest tests/test_*.py` if tests exist for this extension
5. Register the extension in the project README if applicable

### Gala Language Keywords

Keywords: `let`, `mut`, `fn`, `if`, `else`, `match`, `while`, `for`, `in`, `return`, `true`, `false`, `unit`, `qbit`, `measure`, `hadamard`, `cnot`, `phase`, `toffoli`, `uncompute`, `property`

Types: `Bool`, `Int`, `Float`, `String`, `Qbit`, `Unit`

Attributes: `#[property(unitary)]`, `#[property(reversible)]`, `#[property(uncomputes)]`, `#[property(grad_matches)]`
