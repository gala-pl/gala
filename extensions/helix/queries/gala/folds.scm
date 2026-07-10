; Gala fold queries for Helix

(block) @fold
(comment) @fold
(string_literal) @fold

(function_definition
  body: (block)) @fold

(struct_definition) @fold
(enum_definition) @fold
(trait_definition) @fold
(impl_block) @fold

(match_expression) @fold