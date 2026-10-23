---
name: gala-extensions
description: Add or update Gala editor extensions — VS Code, Vim, Helix, Zed, IntelliJ, tree-sitter, highlight.js, Prism, bat, Sublime, TextMate, Cursor
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala has editor extensions in `extensions/` for 12+ editors. All follow the same conventions: TextMate scope `source.gala`, file extension `.gala`, tree-sitter grammar as source of truth.

Extension directories:
- `extensions/vscode/` — TextMate grammar, snippets, LSP client, commands
- `extensions/cursor/` — `.cursorrules`, snippets
- `extensions/intellij/` — Kotlin plugin (highlighter, commenter, brace matcher)
- `extensions/vim/` — Syntax, indent, ftplugin, ftdetect
- `extensions/sublime/` — YAML syntax, completions
- `extensions/textmate/` — Standalone `.tmLanguage` plist
- `extensions/tree-sitter-gala/` — `grammar.js` + bindings (Rust/Node/Python/Swift)
- `extensions/highlight-js/` — Web grammar
- `extensions/prism/` — Web grammar
- `extensions/bat/` — Sublime-syntax for `bat` pager
- `extensions/zed/` — Tree-sitter highlights/indents/folds queries
- `extensions/helix/` — Tree-sitter + `languages.toml`

## Common Tasks

### Add keyword / syntax to all extensions
1. Update `extensions/tree-sitter-gala/grammar.js` — **source of truth**
2. Regenerate TextMate: `cd extensions/tree-sitter-gala && npm run build` → `gala.tmLanguage.json`
3. Copy to `extensions/textmate/`, `extensions/vscode/syntaxes/`
4. Update Vim: `extensions/vim/syntax/gala.vim`
5. Update Sublime: `extensions/sublime/Gala.sublime-syntax`
6. Update highlight.js: `extensions/highlight-js/gala.js`
7. Update Prism: `extensions/prism/components/prism-gala.js`
8. Update bat: `extensions/bat/Gala.sublime-syntax`
9. Update Zed: `extensions/zed/gala.scm` (highlights/indents/folds)
10. Update Helix: `extensions/helix/queries/` (highlights/indents/folds)
11. Update IntelliJ: `extensions/intellij/src/main/kotlin/.../GalaHighlightingLexer.kt`
12. Update Cursor: `extensions/cursor/snippets.gala`
13. Test each extension (see below)

### Test extensions
```bash
# Tree-sitter
cd extensions/tree-sitter-gala && npm test

# VS Code
code --install-extension extensions/vscode/
# Open .gala file, check highlighting, LSP

# Vim
cp extensions/vim/syntax/gala.vim ~/.vim/syntax/
cp extensions/vim/ftdetect/gala.vim ~/.vim/ftdetect/
# Open .gala file

# Helix
# Add to ~/.config/helix/languages.toml (see extensions/README.md)

# Zed
# Add to Zed settings (see extensions/README.md)

# bat
bat cache --build
# Or symlink (see extensions/README.md)
```

### Add snippet / completion
1. VS Code: `extensions/vscode/snippets/gala.json`
2. Cursor: `extensions/cursor/snippets.gala`
3. Sublime: `extensions/sublime/completions.gala.sublime-completions`
4. IntelliJ: `extensions/intellij/resources/liveTemplates/Gala.xml`

### Update LSP integration
- VS Code: `extensions/vscode/package.json` → `languageServer` config
- Assumes `gala-lsp` on `$PATH` (built with compiler)

## Conventions

- **Scope name**: `source.gala` (all TextMate-based)
- **File extension**: `.gala`
- **Keywords** (from LANGUAGE_SPEC.md §2, matching `extensions/vim/syntax/gala.vim`):
  - Control: `fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `while`, `return`
  - Declarations: `import`, `type`, `struct`, `enum`, `trait`, `impl`, `const`, `where`, `as`
  - Effects: `pure`, `quantum`, `prob`
  - Quantum: `qubit`, `qubits`, `measure`, `reverse`, `adjoint`, `control`, `grad`, `drop`
- **Types**: `Qubit`, `Qubits<N>`, `Measured<T>`, `Bool`, `Int`, `Float`, `Complex`, `Params`
- **Effects**: `pure`, `quantum`, `prob`
- **Built-in gates**: `h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `cz`, `swap`
- **Attributes**: `#[property(unitary)]`, `#[property(reversible)]`, `#[property(uncomputes)]`, `#[property(grad_matches)]`
- **Error codes**: `E01xx`–`E06xx` (for diagnostic highlighting)

## Debugging

```bash
# Rebuild tree-sitter grammar
cd extensions/tree-sitter-gala && npm run build

# Test tree-sitter parsing
cd extensions/tree-sitter-gala && echo 'fn main() { h(q) }' | npx tree-sitter parse -

# Check TextMate grammar
plutil -lint extensions/textmate/Gala.tmLanguage

# Verify VS Code extension
code --install-extension extensions/vscode/ --force
```