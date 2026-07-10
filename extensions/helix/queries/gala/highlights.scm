; Gala highlights for Helix

; Comments
(comment) @comment
(doc_comment) @comment

; Keywords
[
  "fn" "let" "mut" "if" "else" "match" "for" "in" "while" "return"
  "import" "type" "struct" "enum" "trait" "impl" "const" "where"
] @keyword

; Effects
[
  "pure" "quantum" "prob"
] @keyword

; Quantum operations
[
  "qubit" "qubits" "measure" "reverse" "adjoint" "control" "grad" "drop"
] @keyword.function

; Booleans
[
  "true" "false"
] @constant.builtin

; Types
(primitive_type) @type
(qubit_type) @type
(qubits_type) @type
(measured_type) @type

; Literals
(integer_literal) @number
(float_literal) @number
(complex_literal) @number
(string_literal) @string

; Operators
[
  "+" "-" "*" "/" "%" "==" "!=" "<" "<=" ">" ">=" "&&" "||" "!" "=" "->" ".." "|"
] @operator

; Punctuation
[
  "{" "}" "(" ")" "[" "]"
] @punctuation

[
  "," ";" ":" "."
] @punctuation.delimiter

; Function definitions
(function_definition
  name: (identifier) @function)

; Identifiers
(identifier) @variable