# tree-sitter-gala

[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for the Gala programming language. Provides incremental, error-tolerant parsing for syntax highlighting, code navigation, and IDE features across all editors that support Tree-sitter.

## Overview

This grammar is the single source of truth for Gala syntax across editors. It is automatically generated from `grammar.js` and produces bindings for Rust, Node.js, Python, and Swift.

## Grammar Coverage

The grammar defines 361 rules covering the full Gala language:

### Tokens
- **Comments:** line (`//`), block (`/* */`), doc comments (`///`, `//!`)
- **Identifiers:** standard, uppercase type names
- **Literals:** integers (decimal, hex, binary), floats, complex numbers, strings (escaped), booleans, unit

### Expressions
- Binary operators (arithmetic, comparison, logical)
- Unary operators (negation, not)
- Function calls, if/match expressions
- Loops (for, while)
- Blocks, tuples, arrays
- Lambda expressions
- Field access, indexing
- Ranges

### Statements
- `let` bindings (mutable, immutable)
- `return` statements

### Types
- Primitive types: `Bool`, `Int`, `Float`, `Complex`, `String`, `Unit`
- Quantum types: `Qubit`, `Qubits<N>`, `Measured<T>`
- Function types, tuple types, array types
- Generic type parameters
- Effect annotations: `pure`, `quantum`, `prob`

### Declarations
- Functions, structs, enums (with variants)
- Traits, trait implementations
- Type aliases, constants
- Imports
- `impl` blocks
- Attributes (`#[...]`)

### Quantum-Specific
- Gate applications: `h`, `x`, `y`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `cz`, `swap`
- `measure` expressions
- `reverse`, `adjoint`, `control` modifiers
- `grad` for automatic differentiation
- `qubit` / `qubits` declarations
- `drop` for explicit qubit release

## Structure

```
tree-sitter-gala/
├── grammar.js              # Grammar definition (source of truth)
├── Cargo.toml              # Rust crate for native bindings
├── package.json            # npm package
├── src/
│   ├── parser.c            # Generated parser
│   └── scanner.c           # Custom scanner (if needed)
└── bindings/
    ├── rust/
    │   └── lib.rs          # Rust binding
    ├── node/
    │   └── index.js        # Node.js binding
    ├── python/
    │   └── tree_sitter_gala/  # Python binding
    └── swift/
        └── TreeSitterGala/    # Swift binding
```

## Usage

### CLI

```bash
# Generate the parser from grammar.js
npx tree-sitter generate

# Test the grammar against corpus files
npx tree-sitter test

# Parse a file and view the syntax tree
npx tree-sitter parse examples/hello_world.gala

# Build WASM grammar for web usage
npx tree-sitter build --wasm
```

### Rust

```toml
[dependencies]
tree-sitter = "0.21"
tree-sitter-gala = { path = "extensions/tree-sitter-gala" }
```

```rust
use tree_sitter::{Parser, Language};

let mut parser = Parser::new();
parser.set_language(&tree_sitter_gala::LANGUAGE.into()).unwrap();

let tree = parser.parse("fn main() -> Int { return 42; }", None).unwrap();
let root = tree.root_node();
assert_eq!(root.kind(), "source_file");
```

### Node.js

```javascript
const Parser = require("tree-sitter");
const Gala = require("tree-sitter-gala");

const parser = new Parser();
parser.setLanguage(Gala);

const tree = parser.parse("fn main() -> Int { return 42; }");
console.log(tree.rootNode.toString());
```

### Python

```python
from tree_sitter import Language, Parser

GALA_LANGUAGE = Language("build/tree-sitter-gala.so", "gala")
parser = Parser()
parser.set_language(GALA_LANGUAGE)

tree = parser.parse(bytes("fn main() -> Int { return 42; }", "utf-8"))
print(tree.root_node.sexp())
```

## Editor Integration

Editors that use Tree-sitter integrate this grammar directly:

| Editor | Integration Point |
|--------|-------------------|
| **Helix** | `extensions/helix/queries/gala/` — highlights, indents, folds |
| **Zed** | `extensions/zed/gala.scm` — Tree-sitter queries |
| **Neovim** | `extensions/vim/` — or directly via nvim-treesitter |
| **VS Code** | Uses TextMate grammar (generated from this grammar) |

## Development

```bash
# After modifying grammar.js, regenerate:
npx tree-sitter generate

# Run the test corpus:
npx tree-sitter test

# Update test fixtures:
# Add test cases to corpus/ directory
```

## Dependencies

- **Rust:** `tree-sitter >= 0.21` (runtime), `cc` (build)
- **Node.js:** `tree-sitter-cli` (dev), `nan` (bindings)

## License

Apache-2.0
