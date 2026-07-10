//! Tree-sitter grammar for the Gala programming language.
//!
//! This crate provides a Rust binding to the tree-sitter Gala parser.
//! The parser C source is generated from `grammar.js` using `tree-sitter generate`.

use tree_sitter::Language;

extern "C" {
    fn tree_sitter_gala() -> Language;
}

pub fn language() -> Language {
    unsafe { tree_sitter_gala() }
}

pub fn language_name() -> &'static str {
    "gala"
}

pub fn node_kind_names() -> &'static [&'static str] {
    &[
        "source_file",
        "comment",
        "doc_comment",
        "identifier",
        "integer_literal",
        "float_literal",
        "complex_literal",
        "string_literal",
        "boolean_literal",
        "unit_literal",
        "expression",
        "binary_expression",
        "unary_expression",
        "call_expression",
        "if_expression",
        "match_expression",
        "for_expression",
        "while_expression",
        "block",
        "tuple_expression",
        "array_expression",
        "lambda_expression",
        "field_expression",
        "index_expression",
        "let_statement",
        "return_statement",
        "function_definition",
        "struct_definition",
        "enum_definition",
        "trait_definition",
        "impl_block",
        "type_alias",
        "const_definition",
        "import_statement",
        "type",
        "primitive_type",
        "qubit_type",
        "qubits_type",
        "measured_type",
        "fn_type",
        "tuple_type",
        "array_type",
        "generic_type",
    ]
}

pub fn grammar_json_content() -> &'static str {
    include_str!("../../grammar.js")
}

/// Highlight query for Gala (standard capture names for tree-sitter highlighting).
pub fn highlights_query() -> &'static str {
    ";; Gala highlight queries — standard tree-sitter capture groups
[
  (comment)
  (doc_comment)
] @comment

(doc_comment) @string.special

[
  \"fn\" \"let\" \"mut\" \"if\" \"else\" \"match\" \"for\" \"in\" \"while\" \"return\"
  \"import\" \"type\" \"struct\" \"enum\" \"trait\" \"impl\" \"const\" \"where\"
] @keyword

[
  \"pure\" \"quantum\" \"prob\"
] @keyword.storage

[
  \"qubit\" \"qubits\" \"measure\" \"reverse\" \"adjoint\" \"control\" \"grad\" \"drop\"
] @keyword.function

[
  \"true\" \"false\"
] @boolean

[
  (primitive_type)
  (qubit_type)
  (qubits_type)
  (measured_type)
] @type.builtin

(generic_type
  identifier: (identifier) @type)

[
  (integer_literal)
  (float_literal)
  (complex_literal)
] @number

(string_literal) @string

(comment) @comment
(doc_comment) @comment.documentation

[
  \"+\" \"-\" \"*\" \"/\" \"%\" \"==\" \"!=\" \"<\" \"<=\" \">\" \">=\" \"&&\" \"||\" \"!\" \"=\" \"->\" \"..\" \"|\"
] @operator

[
  \"{\" \"}\" \"(\" \")\" \"[\" \"]\"
] @punctuation.bracket

[
  \",\" \";\" \":\" \".\"
] @punctuation.delimiter

(call_expression
  function: (identifier) @function)

(function_definition
  name: (identifier) @function)
"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let language = language();
        assert_eq!(language.name(), "gala");
    }
}