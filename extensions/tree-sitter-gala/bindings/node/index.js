const Parser = require('tree-sitter');

function load() {
  const parser = new Parser();
  const Gala = require('./gala');
  parser.setLanguage(Gala);
  return parser;
}

module.exports = { load };