# Gala IDE Extensions & Syntax Highlighters

Editor integrations and syntax highlighting for the **Gala programming language** — a hybrid
quantum-classical language with first-class differentiation, linear types for no-cloning, and a
checked effect system.

## Quick Start

### VS Code

```bash
code --install-extension extensions/vscode/
# Or package: cd extensions/vscode && vsce package
```

Opens `.gala` files with full syntax highlighting, code snippets, and an LSP client that connects
to `gala-lsp`.

### Vim / Neovim

```vim
" Add to ~/.vimrc or ~/.config/nvim/init.vim
set runtimepath^=/path/to/gala/extensions/vim
" Or copy the files:
"   cp extensions/vim/syntax/gala.vim ~/.vim/syntax/
"   cp extensions/vim/ftdetect/gala.vim ~/.vim/ftdetect/
"   cp extensions/vim/indent/gala.vim ~/.vim/indent/
"   cp extensions/vim/ftplugin/gala.vim ~/.vim/ftplugin/
```

### Helix

Add to your `~/.config/helix/languages.toml`:

```toml
[language]
name = "gala"
scope = "source.gala"
file-types = ["gala"]
roots = ["gala.toml"]
comment-token = "//"
block-comment-tokens = { start = "/*", end = "*/" }
```

### Zed

In Zed settings, add:

```json
{
  "languages": {
    "Gala": {
      "scope": "source.gala",
      "file_extensions": ["gala"],
      "grammar": {
        "path": "/path/to/gala/extensions/zed/gala.scm"
      }
    }
  }
}
```

### Sublime Text

Copy to `~/Library/Application Support/Sublime Text/Packages/User/`:

```bash
cp extensions/sublime/*.sublime-* ~/Library/Application\ Support/Sublime\ Text/Packages/User/
```

### IntelliJ IDEA / PyCharm / CLion

Open the `extensions/intellij/` directory as a project and build via Gradle, or install the
prebuilt plugin from the marketplace.

### `bat` (syntax-aware pager)

```bash
bat cache --build
# Or symlink:
mkdir -p "$(bat --config-dir)/syntaxes"
ln -s /path/to/gala/extensions/bat/Gala.sublime-syntax "$(bat --config-dir)/syntaxes/"
```

### Web pages (highlight.js / Prism.js)

```html
<script src="/path/to/extensions/highlight-js/gala.js"></script>
<script>hljs.highlightAll();</script>
```

## Extension Overview

| Directory | Editor / Tool | Features |
|-----------|--------------|----------|
| `vscode/` | VS Code | TextMate grammar, snippets, language config, LSP client, commands |
| `cursor/` | Cursor IDE | `.cursorrules`, snippets |
| `intellij/` | IntelliJ IDEA | Kotlin plugin: highlighter, commenter, brace matcher, color scheme |
| `vim/` | Vim / Neovim | Syntax, indent, ftplugin, ftdetect |
| `sublime/` | Sublime Text | YAML-based syntax, completions, settings |
| `textmate/` | TextMate / generic | Standalone `.tmLanguage` plist grammar |
| `tree-sitter-gala/` | Tree-sitter | `grammar.js` + Rust/Node/Python/Swift bindings |
| `highlight-js/` | highlight.js | Grammar for web-based code blocks |
| `prism/` | Prism.js | Grammar for web-based code blocks |
| `bat/` | `bat` pager | Sublime-syntax grammar for the `bat` syntax-aware pager |
| `zed/` | Zed editor | Tree-sitter highlights/indents/folds queries |
| `helix/` | Helix editor | Tree-sitter highlights/indents/folds + `languages.toml` |

## Language Server

The extensions assume `gala-lsp` is installed and available on `$PATH`. The LSP is built as part
of the Gala compiler toolchain (see `docs/ARCHITECTURE.md` and `tools/gala-lsp/`). It shares the
`salsa` query graph with the compiler, providing live diagnostics, hover, go-to-definition, and
inline circuit diagrams.

## Architecture

- **Scope name:** `source.gala` (TextMate-based editors)
- **File extension:** `.gala`
- **Keywords:** `fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `while`, `return`,
  `import`, `type`, `struct`, `enum`, `trait`, `impl`, `const`, `where`
- **Reserved quantum:** `qubit`, `qubits`, `measure`, `reverse`, `adjoint`, `control`, `grad`, `drop`
- **Types:** `Qubit`, `Qubits<N>`, `Measured<T>`, `Bool`, `Int`, `Float`, `Complex`, `Vec<T>`,
  `Params`, `String`, `Unit`
- **Effects:** `pure`, `quantum`, `prob`
- **Built-in gates:** `h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `cz`, `swap`
- **Error codes:** `E01xx` (syntax), `E02xx` (types), `E03xx` (effects), `E04xx` (linearity),
  `E05xx` (uncomputation)

## Contributing

See `docs/CONTRIBUTING.md` and `docs/TOOLCHAIN_DX.md` in the repository root. All extensions follow
the same conventions: TextMate grammars use `source.gala` scope, tree-sitter grammars reference
the single source of truth in `tree-sitter-gala/grammar.js`, and Vim syntax files follow Vim's
standard group-name conventions.

## License

Apache-2.0 — see `LICENSE.md` in the repository root.