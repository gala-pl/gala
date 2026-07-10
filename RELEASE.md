# Gala Release Notes

## 0.1.0 (TBD)

Initial release.

### Highlights

- First working version of the Gala compiler pipeline: lexer → parser → AST → IR → codegen
- Core runtime (`gala-core`): `Int`, `Float`, `Bool`, `Tuple` primitives
- Standard library scaffolding: `gala-string`, `gala-io`, `gala-collections`
- Runtime components: channels, region-based allocation, async executor, GC, Python bridge
- Toolchain: CLI (`gala-cli`), formatter (`gala-fmt`), linter (`gala-lint`), LSP (`gala-lsp`), package manager (`gala-pkg`)
- Documentation site powered by Next.js 16 + Fumadocs with MDX content, full-text search, and dark mode
- Editor extensions for VS Code, Vim, Helix, Zed, IntelliJ, Sublime Text, and more
- Tree-sitter grammar with 361 rules and Rust/Node/Python/Swift bindings

### Known Issues

- The LSP is a scaffold; full IDE features are in development
- The standard library (`gala-std`) is minimal; core types and collections are functional
- Quantum features (qubits, gates, effects, uncomputation) are designed but not yet implemented in the compiler
