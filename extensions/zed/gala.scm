; Gala highlights for Zed
; Uses tree-sitter capture groups

; Comments
(comment) @comment
(doc_comment) @comment.doc

; Keywords
[
  "fn" "let" "mut" "if" "else" "match" "for" "in" "while" "return"
  "import" "type" "struct" "enum" "trait" "impl" "const" "where"
] @keyword

; Effects
[
  "pure" "quantum" "prob"
] @keyword.important

; Quantum operations
[
  "qubit" "qubits" "measure" "reverse" "adjoint" "control" "grad" "drop"
] @function.special

; Booleans
[
  "true" "false"
] @boolean

; Types
(primitive_type) @type
(qubit_type) @type.quantum
(qubits_type) @type.quantum
(measured_type) @type.quantum
(generic_type
  name: (identifier) @type)

; Literals
(integer_literal) @number
(float_literal) @number.float
(complex_literal) @number.complex
(string_literal) @string

; Operators
[
  "+" "-" "*" "/" "%" "==" "!=" "<" "<=" ">" ">=" "&&" "||" "!" "=" "->" ".." "|"
] @operator

; Punctuation
[
  "{" "}" "(" ")" "[" "]"
] @punctuation.bracket

[
  "," ";" ":" "."
] @punctuation.delimiter

; Function calls
(call_expression
  function: (identifier) @function)

; Function definitions
(function_definition
  name: (identifier) @function.definition)

; Identifiers
(identifier) @variable