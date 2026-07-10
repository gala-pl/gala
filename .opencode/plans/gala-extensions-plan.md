# Gala IDE Extensions & Syntax Highlighters — Build Plan

## Goal

Generate production-quality IDE extensions, editor integrations, and syntax highlighters for the Gala
programming language under `/Users/gabrielfonseca/Downloads/gala/extensions/`.

## Language Reference (from docs/)

Gala is a **hybrid quantum-classical language** — expression-oriented, braces, `fn`, `let`, type
inference, linear types for qubits, effect system (`pure`/`quantum`/`prob`), native `grad`.

**Keywords:** `fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `while`, `return`, `import`,
`type`, `struct`, `enum`, `trait`, `impl`, `pure`, `quantum`, `prob`, `const`, `true`, `false`

**Reserved quantum keywords:** `qubit`, `qubits`, `measure`, `reverse`, `adjoint`, `control`,
`grad`, `drop`

**Types:** `Qubit`, `Qubits<N>`, `Measured<T>`, `Bool`, `Int`, `Float`, `Complex`, `Vec<T>`,
`Params`, `Unit` — plus user-defined structs/enums

**Built-in gates:** `h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `cz`, `swap`

**Literals:** integers (`42`, `0xFF`, `0b1010`), floats (`3.14`, `1e-3`), complex (`2+3i`),
bools, strings, unit `()`

**Comments:** `//`, `/* */` (nestable), `///` doc comments

**Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!`,
`->`, `|`, `..`

**Error codes:** `E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity, `E05xx` uncomputation

**Effects on functions:** `fn foo() -> T pure`, `fn bar() -> Measured<Bool> prob`

## Deliverables

### 1. VS Code Extension (`extensions/vscode/`)
- `package.json` — language id `gala`, `.gala` extension, activation events, commands
- `syntaxes/gala.tmLanguage.json` — comprehensive TextMate grammar covering all tokens,
  types, quantum keywords, effects, comments, strings, operators, doc comments with `///` captures
- `language-configuration.json` — comment toggling, bracket matching, auto-closing pairs,
  surrounding pairs, word pattern, indentation rules
- `snippets/gala.json` — code snippets for:
  - `fn` → function definition boilerplate
  - `bell` → Bell pair circuit
  - `quantum` → quantum effect function
  - `prob` → probabilistic function with measurement
  - `for` → quantum for loop
  - `if` → if expression
  - `match` → match expression
  - `measure` → measurement with result capture
  - `qft` → QFT algorithm pattern
  - `grad` → gradient declaration
  - `struct` → struct definition
  - `import` → import statement
  - `let` → variable binding
- `client/extension.js` — LSP client activation stub, commands for check/run/explain/circuit

### 2. Cursor IDE (`extensions/cursor/`)
- `.cursorrules` — Cursor-specific rules: context about Gala (quantum types, effect system,
  linearity, `gala` commands)
- `snippets/gala.json` — same snippet set adapted for Cursor

### 3. IntelliJ IDEA Plugin (`extensions/intellij/`)
- `build.gradle.kts` — Gradle build config with intellij plugin
- `src/main/resources/META-INF/plugin.xml` — plugin descriptor: id, name, vendor, dependencies,
  language definitions, annotations, codeInsight
- `src/main/kotlin/com/gala/lang/GalaLanguage.kt` — language definition
- `src/main/kotlin/com/gala/lang/GalaSyntaxHighlighter.kt` — syntax highlighter with TextMate-ish
  coloring: keywords, types, effects, built-ins, strings, comments, numbers, operators
- `src/main/kotlin/com/gala/lang/GalaCommenter.kt` — line/block commenter
- `src/main/kotlin/com/gala/lang/GalaBraceMatcher.kt` — brace matching
- `src/main/resources/colorSchemes/GalaDefault.xml` — default color scheme
- Grammar file as `.bnf` for PSI-based highlighting

### 4. Vim/Neovim (`extensions/vim/`)
- `ftdetect/gala.vim` — `au BufNewFile,BufRead *.gala setf gala`
- `syntax/gala.vim` — syntax highlighting groups:
  - `galaKeyword` — `fn`, `let`, `if`, `else`, `match`, `for`, `while`, `return`, `import`,
    `type`, `struct`, `enum`, `trait`, `impl`, `const`, `mut`, `in`, `where`
  - `galaEffect` — `pure`, `quantum`, `prob`
  - `galaQuantum` — `qubit`, `qubits`, `measure`, `reverse`, `adjoint`, `control`, `grad`, `drop`
  - `galaType` — built-in types
  - `galaGate` — `h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `cz`, `swap`
  - `galaComment` — `//`, `/* */`, `///`
  - `galaString` — string literals
  - `galaNumber` — numeric literals
  - `galaOperator` — operators
  - `galaFunction` / `galaIdentifier` / `galaDelimiter`
- `indent/gala.vim` — `cindent`-based indentation with braces
- `ftplugin/gala.vim` — filetype settings: `commentstring`, `path`, `compiler`, `makeprg`
- `after/syntax/gala.vim` — additional syntax refinements, error highlight groups

### 5. Sublime Text (`extensions/sublime/`)
- `Gala.sublime-syntax` — YAML-based syntax definition with contexts:
  - `main` → comments, doc comments, strings, block comments
  - `block-comment` → nestable `/* */`
  - `string-content` → double-quoted strings
  - `function-definition` → `fn` pattern
  - `expression` → operators, precedence
- `Gala.sublime-completions` — tab-completion for all keywords, types, gates
- `Preferences.sublime-settings` — ruler, tab size, comment rules

### 6. Standalone TextMate Grammar (`extensions/textmate/`)
- `Gala.tmLanguage` — pure XML `.plist` TextMate grammar (SCTable format)
- Can be consumed by any editor supporting TextMate grammars (VS Code already has its own copy)

### 7. Tree-sitter Grammar (`extensions/tree-sitter-gala/`)
- `grammar.js` — complete tree-sitter grammar with:
  - Token nodes: `comment`, `doc_comment`, `block_comment`, integers, floats, complex, strings
  - Expression nodes: `binary_expression`, `unary_expression`, `call_expression`,
    `if_expression`, `match_expression`, `for_expression`, `while_expression`,
    `block`, `tuple_expression`, `array_expression`, `lambda_expression`
  - Statement nodes: `let_statement`, `return_statement`, `expression_statement`
  - Declaration nodes: `function_definition`, `struct_definition`, `enum_definition`,
    `trait_definition`, `impl_block`, `type_alias`, `const_definition`
  - Type nodes: `type`, `qubits_type`, `qubit_type`, `measured_type`, `fn_type`,
    `tuple_type`, `array_type`
  - Effect nodes: `effect`
  - Import nodes: `import_statement`
  - Conflicts resolved via precedence and associativity
- `package.json` — node package for tree-sitter
- `bindings/` — Rust, Node, Python, Swift bindings
- `Cargo.toml` — Rust crate for the parser

### 8. highlight.js Grammar (`extensions/highlight-js/`)
- `gala.js` — complete highlight.js language definition:
  - `keywords` array: by category (keyword, type, effect, quantum, literal, built_in)
  - `contains`: comments, strings, numbers, function declarations
  - Auto-detection via `.gala` extension

### 9. Prism.js Grammar (`extensions/prism/`)
- `gala.js` — Prism language definition:
  - Prism token array for each pattern class
  - `keyword`, `builtin`, `type`, `function`, `string`, `number`, `operator`,
    `punctuation`, `comment`, `doc-comment`
  - Grammar aliases, auto-linking

### 10. `bat` Syntax Highlighting (`extensions/bat/`)
- `Gala.sublime-syntax` — identical sublime-syntax adapted for `bat` usage

### 11. Zed Editor (`extensions/zed/`)
- `gala.scm` — tree-sitter queries for:
  - `highlights.scm` pattern queries for each capture group
  - `indents.scm` for automatic indentation
  - `folds.scm` for folding regions (braces, comments)

### 12. Helix Editor (`extensions/helix/`)
- `languages.toml` — language server config, file types, `||` as auto-pairs, block comment,
  line comment, auto-format command
- `queries/gala/highlights.scm` — tree-sitter highlight queries
- `queries/gala/indents.scm` — tree-sitter indent queries
- `queries/gala/folds.scm` — tree-sitter fold queries

### 13. Root README (`extensions/README.md`)
- Overview of all available extensions
- Quick-start per editor
- Links to the LSP (`gala-lsp`)
- Reference: `docs/LANGUAGE_SPEC.md`, `docs/TYPE_SYSTEM.md`, `docs/TOOLCHAIN_DX.md`

## Implementation Order

1. VS Code extension (most important — directly referenced in ROADMAP.md WP-035)
2. TextMate grammar (source of truth for many editors)
3. Tree-sitter grammar (language-agnostic parser)
4. Vim/Neovim (community standard)
5. Cursor IDE
6. IntelliJ IDEA (enterprise users)
7. Sublime Text
8. highlight.js + Prism.js (documentation website)
9. Zed + Helix (modern editors)
10. bat
11. Root README

## Key Design Decisions

- **Scope name:** `source.gala` for TextMate-based editors
- **File extension:** `.gala` (per LANGUAGE_SPEC.md)
- **Tree-sitter:** Single `grammar.js` generates all language bindings
- **Vim syntax groups:** Follow Vim's `:help group-name` conventions for compatibility with
  popular colorschemes
- **VS Code LSP client:** Stub that will connect to `gala-lsp` once that crate is built
  (per architecture, the LSP shares the `salsa` query graph)
- **Diagnostic codes:** `E01xx`-`E05xx` ranges should be linked in hover providers once the LSP
  is available
- **Quantum keyword coloring:** Gates (`h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`,
  `cz`, `swap`) get a distinct token class so themes can color them differently from regular
  built-ins

## Files to Create

Total: ~40 files

| Editor | Files |
|--------|-------|
| VS Code | `package.json`, `syntaxes/gala.tmLanguage.json`, `language-configuration.json`, `snippets/gala.json`, `client/extension.js` |
| Cursor | `.cursorrules`, `snippets/gala.json` |
| IntelliJ | `build.gradle.kts`, `plugin.xml`, `GalaLanguage.kt`, `GalaSyntaxHighlighter.kt`, `GalaCommenter.kt`, `GalaBraceMatcher.kt`, `colorSchemes/GalaDefault.xml` |
| Vim | `ftdetect/gala.vim`, `syntax/gala.vim`, `indent/gala.vim`, `ftplugin/gala.vim`, `after/syntax/gala.vim` |
| Sublime | `Gala.sublime-syntax`, `Gala.sublime-completions`, `Preferences.sublime-settings` |
| TextMate | `Gala.tmLanguage` |
| Tree-sitter | `grammar.js`, `package.json`, `Cargo.toml`, `bindings/rust/lib.rs`, `bindings/node/index.js`, `bindings/python/tree_sitter_gala/__init__.py`, `bindings/swift/TreeSitterGala/TreeSitterGala.swift` |
| highlight.js | `gala.js` |
| Prism | `gala.js` |
| bat | `Gala.sublime-syntax` |
| Zed | `gala.scm` |
| Helix | `languages.toml`, `queries/gala/highlights.scm`, `queries/gala/indents.scm`, `queries/gala/folds.scm` |
| Root | `README.md` |