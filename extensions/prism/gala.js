(function (Prism) {
  Prism.languages.gala = Prism.languages.extend('clike', {
    'comment': [
      {
        pattern: /(^|[^\\])\/\/\/.*/,
        lookbehind: true,
        greedy: true,
        alias: 'doc-comment'
      },
      {
        pattern: /(^|[^\\])\/\/.*/,
        lookbehind: true,
        greedy: true
      },
      {
        pattern: /\/\*[\s\S]*?\*\//,
        greedy: true
      }
    ],
    'string': {
      pattern: /"(?:[^"\\]|\\.)*"/,
      greedy: true
    },
    'keyword': [
      {
        pattern: /\b(fn|let|mut|if|else|match|for|in|while|return|import|from|as|type|struct|enum|trait|impl|const|where)\b/,
        alias: 'keyword'
      },
      {
        pattern: /\b(pure|quantum|prob)\b/,
        alias: 'effect'
      },
      {
        pattern: /\b(qubit|qubits|measure|reverse|adjoint|control|grad|drop)\b/,
        alias: 'quantum'
      }
    ],
    'builtin': [
      {
        pattern: /\b(h|x|y|z|s|t|rx|ry|rz|cx|cz|swap)\b/,
        alias: 'gate'
      },
      {
        pattern: /\b(print|println|assert|len|sample|expectation|distribution|backend)\b/,
        alias: 'builtin'
      }
    ],
    'type': [
      {
        pattern: /\b(Qubit|Qubits|Measured|Bool|Int|Float|Complex|Vec|Params|String|Unit|Self)\b/,
        alias: 'type'
      }
    ],
    'boolean': /\b(true|false)\b/,
    'operator': /[+\-*/%]=?|==?|!=?|<=?|>=?|&&|\|\|?|!|->|\.\.|::?/,
    'punctuation': /[{}()\[\],;:]/
  });

  Prism.languages.gala['number'] = [
    /\b\d+[iIjJ]\b/,
    /\b\d+\.\d*([eE][+-]?\d+)?\b/,
    /\b0[xX][0-9a-fA-F]+\b/,
    /\b0[bB][01]+\b/,
    /\b\d+\b/
  ];

  if (Prism.languages.markup) {
    Prism.languages.insertBefore('gala', 'comment', {
      'doc-comment': {
        pattern: /\/\/\/.*/,
        greedy: true,
        alias: 'doc-comment'
      }
    });
  }
})(Prism);