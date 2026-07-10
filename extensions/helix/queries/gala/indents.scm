; Gala indent queries for Helix

(block) @indent

(block
  "}" @outdent)

(function_definition) @indent

(function_definition
  body: (block) @indent)

(if_expression
  alternative: (block) @indent)

(for_expression
  body: (block) @indent)

(while_expression
  body: (block) @indent)