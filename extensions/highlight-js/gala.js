/*
Language: Gala
Description: Hybrid quantum-classical programming language with first-class differentiation
Category: quantum, scientific, ml
Author: Gala Language Team
Website: https://gala-lang.dev
*/

export default function(hljs) {
  const GALA_KEYWORDS = {
    $pattern: /[A-Za-z_α-ψ][A-Za-z0-9_α-ψ]*/,
    keyword: [
      'fn', 'let', 'mut', 'if', 'else', 'match', 'for', 'in',
      'while', 'return', 'import', 'from', 'as', 'type', 'struct',
      'enum', 'trait', 'impl', 'const', 'where'
    ],
    type: [
      'Qubit', 'Qubits', 'Measured', 'Bool', 'Int', 'Float', 'Complex',
      'Vec', 'Params', 'String', 'Unit', 'Self'
    ],
    effect: [
      'pure', 'quantum', 'prob'
    ],
    quantum: [
      'qubit', 'qubits', 'measure', 'reverse', 'adjoint', 'control', 'grad', 'drop'
    ],
    literal: [
      'true', 'false'
    ],
    built_in: [
      'h', 'x', 'y', 'z', 's', 't', 'rx', 'ry', 'rz', 'cx', 'cz', 'swap',
      'print', 'println', 'assert', 'len', 'sample', 'expectation', 'distribution',
      'backend'
    ]
  };

  const COMPLEX_NUMBER = {
    className: 'number',
    begin: /\b\d+[iIjJ]\b/,
    relevance: 2
  };

  const FLOAT_NUMBER = {
    className: 'number',
    begin: /\b\d+\.\d*([eE][+-]?\d+)?\b/
  };

  const HEX_NUMBER = {
    className: 'number',
    begin: /\b0[xX][0-9a-fA-F]+\b/
  };

  const BIN_NUMBER = {
    className: 'number',
    begin: /\b0[bB][01]+\b/
  };

  const INTEGER = {
    className: 'number',
    begin: /\b\d+\b/
  };

  const STRING = {
    className: 'string',
    begin: /"/,
    end: /"/,
    contains: [{ className: 'escape', begin: /\\./ }]
  };

  const LINE_COMMENT = {
    className: 'comment',
    begin: '///',
    end: '$',
    relevance: 3
  };

  const BLOCK_COMMENT = {
    className: 'comment',
    begin: '/\\*',
    end: '\\*/',
    contains: ['self']
  };

  const LINE_COMMENT2 = {
    className: 'comment',
    begin: '//',
    end: '$'
  };

  return {
    name: 'Gala',
    aliases: ['gala'],
    keywords: GALA_KEYWORDS,
    contains: [
      COMPLEX_NUMBER,
      FLOAT_NUMBER,
      HEX_NUMBER,
      BIN_NUMBER,
      INTEGER,
      STRING,
      LINE_COMMENT,
      BLOCK_COMMENT,
      LINE_COMMENT2,
      {
        className: 'function',
        beginKeywords: 'fn',
        end: /[({]/,
        excludeEnd: true,
        contains: [
          {
            className: 'title',
            begin: /[A-Za-z_][A-Za-z0-9_]*/,
            relevance: 0
          },
          {
            className: 'params',
            begin: /\(/,
            end: /\)/,
            excludeBegin: true,
            excludeEnd: true,
            keywords: GALA_KEYWORDS,
            contains: [
              STRING,
              LINE_COMMENT,
              LINE_COMMENT2,
              BLOCK_COMMENT
            ]
          }
        ]
      },
      {
        className: 'operator',
        begin: /[+\-*/%=!<>]=?|->|\.\.|[&|]{1,2}/
      },
      {
        className: 'punctuation',
        begin: /[{}()\[\],;:]/
      }
    ]
  };
}