module.exports = grammar({
  name: 'gala',

  extras: $ => [
    /\s/,
    $.comment,
    $.doc_comment,
  ],

  conflicts: $ => [
    [$.type, $.expression],
  ],

  inline: () => [
    $._comma_separated_list,
    $._semicolon_list,
  ],

  supertypes: $ => [
    $.expression,
    $.declaration,
    $.type,
    $.pattern,
  ],

  rules: {
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      $.function_definition,
      $.struct_definition,
      $.enum_definition,
      $.trait_definition,
      $.impl_block,
      $.type_alias,
      $.const_definition,
      $.import_statement,
    ),

    // ----- Comments -----
    comment: $ => token(choice(
      seq('//', /[^\n]*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),

    doc_comment: $ => token(seq('///', /[^\n]*/)),

    // ----- Identifiers -----
    identifier: $ => /[A-Za-z_\u0391-\u03C9][A-Za-z0-9_\u0391-\u03C9]*/,

    // ----- Literals -----
    integer_literal: $ => choice(
      token(seq(choice('0x', '0X'), /[0-9a-fA-F]+/)),
      token(seq(choice('0b', '0B'), /[01]+/)),
      token(/\d+/),
    ),

    float_literal: $ => token(/\d+\.\d*([eE][+-]?\d+)?/),

    complex_literal: $ => token(/\d+[iIjJ]/),

    string_literal: $ => token(seq(
      '"',
      repeat(choice(
        /[^"\\]/,
        /\\./,
      )),
      '"',
    )),

    boolean_literal: $ => choice('true', 'false'),

    unit_literal: $ => seq('(', ')'),

    // ----- Expressions -----
    expression: $ => choice(
      $.integer_literal,
      $.float_literal,
      $.complex_literal,
      $.string_literal,
      $.boolean_literal,
      $.unit_literal,
      $.identifier,
      $.binary_expression,
      $.unary_expression,
      $.call_expression,
      $.if_expression,
      $.match_expression,
      $.for_expression,
      $.while_expression,
      $.block,
      $.tuple_expression,
      $.array_expression,
      $.lambda_expression,
      $.field_expression,
      $.index_expression,
    ),

    binary_expression: $ => choice(
      prec.left(1, seq($.expression, choice('||', 'or'), $.expression)),
      prec.left(2, seq($.expression, choice('&&', 'and'), $.expression)),
      prec.left(3, seq($.expression, choice('==', '!='), $.expression)),
      prec.left(4, seq($.expression, choice('<', '<=', '>', '>='), $.expression)),
      prec.left(5, seq($.expression, choice('+', '-'), $.expression)),
      prec.left(6, seq($.expression, choice('*', '/', '%'), $.expression)),
      prec.left(7, seq($.expression, '..', $.expression)),
    ),

    unary_expression: $ => prec(8, seq(
      choice('-', '!'),
      $.expression,
    )),

    call_expression: $ => seq(
      $.expression,
      '(',
      optional(seq($.expression, repeat(seq(',', $.expression)))),
      ')',
    ),

    if_expression: $ => prec(1, seq(
      'if', $.expression, $.block,
      optional(seq('else', choice($.if_expression, $.block))),
    )),

    match_expression: $ => seq(
      'match', $.expression, '{',
      repeat(seq($.pattern, '=>', $.expression, optional(','))),
      '}',
    ),

    for_expression: $ => seq(
      'for', $.pattern, 'in', $.expression, $.block,
    ),

    while_expression: $ => seq(
      'while', $.expression, $.block,
    ),

    block: $ => seq(
      '{',
      repeat($._statement),
      optional($.expression),
      '}',
    ),

    tuple_expression: $ => seq(
      '(',
      seq($.expression, repeat(seq(',', $.expression))),
      optional(','),
      ')',
    ),

    array_expression: $ => seq(
      '[',
      optional(seq($.expression, repeat(seq(',', $.expression)))),
      ']',
    ),

    lambda_expression: $ => seq(
      '|', optional($._params), '|',
      optional(seq('->', $.type)),
      optional(choice('pure', 'quantum', 'prob')),
      choice($.expression, $.block),
    ),

    field_expression: $ => seq(
      $.expression, '.', $.identifier,
    ),

    index_expression: $ => seq(
      $.expression, '[', $.expression, ']',
    ),

    // ----- Statements -----
    _statement: $ => choice(
      $.let_statement,
      $.return_statement,
      seq($.expression, ';'),
    ),

    let_statement: $ => seq(
      'let', optional('mut'), $.pattern,
      optional(seq(':', $.type)),
      optional(seq('=', $.expression)),
      ';',
    ),

    return_statement: $ => seq(
      'return', optional($.expression), ';',
    ),

    // ----- Patterns -----
    pattern: $ => choice(
      $.identifier,
      $.wildcard_pattern,
      $.tuple_pattern,
    ),

    wildcard_pattern: $ => '_',

    tuple_pattern: $ => seq(
      '(',
      seq($.pattern, repeat(seq(',', $.pattern))),
      optional(','),
      ')',
    ),

    // ----- Types -----
    type: $ => choice(
      $.primitive_type,
      $.qubits_type,
      $.qubit_type,
      $.measured_type,
      $.fn_type,
      $.tuple_type,
      $.array_type,
      $.generic_type,
    ),

    primitive_type: $ => choice(
      'Bool', 'Int', 'Float', 'Complex', 'String', 'Unit', 'Params', 'Vec',
    ),

    qubit_type: $ => 'Qubit',

    qubits_type: $ => seq('Qubits', '<', $._const_expr, '>'),

    measured_type: $ => seq('Measured', '<', $.type, '>'),

    fn_type: $ => seq(
      'fn', '<', optional(seq($.type, repeat(seq(',', $.type)))), '>',
      optional(seq('->', $.type)),
      optional(choice('pure', 'quantum', 'prob')),
    ),

    tuple_type: $ => seq(
      '(',
      seq($.type, repeat(seq(',', $.type))),
      ')',
    ),

    array_type: $ => seq(
      '[', $.type, ';', $._const_expr, ']',
    ),

    generic_type: $ => seq(
      $.identifier,
      '<', seq($.type, repeat(seq(',', $.type))), '>',
    ),

    _const_expr: $ => $.integer_literal, // or identifier

    // ----- Declarations -----
    function_definition: $ => seq(
      'fn', $.identifier,
      optional($.generic_parameters),
      '(', optional($._params), ')',
      optional(seq('->', $.type)),
      optional(choice('pure', 'quantum', 'prob')),
      $.block,
    ),

    generic_parameters: $ => seq(
      '<',
      seq($.generic_param, repeat(seq(',', $.generic_param))),
      '>',
    ),

    generic_param: $ => choice(
      seq($.identifier, optional(seq(':', 'type'))),
      seq('const', $.identifier, ':', $.type),
    ),

    _params: $ => seq(
      $.param,
      repeat(seq(',', $.param)),
    ),

    param: $ => seq(
      optional('mut'),
      $.pattern,
      ':', $.type,
    ),

    struct_definition: $ => seq(
      'struct', $.identifier,
      optional($.generic_parameters),
      '{',
      optional(seq($.struct_field, repeat(seq(',', $.struct_field)), optional(','))),
      '}',
    ),

    struct_field: $ => seq(
      $.identifier,
      ':', $.type,
    ),

    enum_definition: $ => seq(
      'enum', $.identifier,
      optional($.generic_parameters),
      '{',
      optional(seq($.enum_variant, repeat(seq(',', $.enum_variant)), optional(','))),
      '}',
    ),

    enum_variant: $ => seq(
      $.identifier,
      optional(seq('(', repeat(seq($.type, repeat(seq(',', $.type)))), ')')),
    ),

    trait_definition: $ => seq(
      'trait', $.identifier,
      optional($.generic_parameters),
      '{',
      repeat($.trait_item),
      '}',
    ),

    trait_item: $ => seq(
      'fn', $.identifier,
      '(', optional($._params), ')',
      optional(seq('->', $.type)),
      optional(choice('pure', 'quantum', 'prob')),
      ';',
    ),

    impl_block: $ => seq(
      'impl', $.type, optional(choice('pure', 'quantum', 'prob')),
      '{',
      repeat($.function_definition),
      '}',
    ),

    type_alias: $ => seq(
      'type', $.identifier,
      optional($.generic_parameters),
      '=', $.type, ';',
    ),

    const_definition: $ => seq(
      'const', $.identifier, ':', $.type, '=', $.expression, ';',
    ),

    import_statement: $ => seq(
      'import', $._import_path,
      optional(seq('as', $.identifier)),
      ';',
    ),

    _import_path: $ => seq(
      $.identifier,
      repeat(seq('.', $.identifier)),
    ),

    path: $ => seq(
      $.identifier,
      repeat(seq('.', $.identifier)),
    ),
  },
});