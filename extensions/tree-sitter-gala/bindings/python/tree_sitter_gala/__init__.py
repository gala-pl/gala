"""Tree-sitter grammar for the Gala programming language."""

from tree_sitter import Language, Parser

def language() -> Language:
    """Load the Gala tree-sitter grammar."""
    return Language("/path/to/tree-sitter-gala/src/parser.c", "gala")

def create_parser() -> Parser:
    """Create a parser for the Gala language."""
    parser = Parser()
    parser.set_language(language())
    return parser