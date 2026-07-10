#!/usr/bin/env python3
"""Tests for the tree-sitter grammar."""

import os
import json
import unittest

TS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "tree-sitter-gala"))
FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")


class TestTreeSitterGrammarJS(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "grammar.js")) as f:
            self.content = f.read()

    def test_grammar_name(self):
        self.assertIn("name: 'gala'", self.content)

    def test_has_all_declaration_types(self):
        decls = ["function_definition", "struct_definition", "enum_definition",
                 "trait_definition", "impl_block", "type_alias", "const_definition",
                 "import_statement"]
        for d in decls:
            self.assertIn(d, self.content, f"Missing declaration: {d}")

    def test_has_all_expressions(self):
        exprs = ["binary_expression", "unary_expression", "call_expression",
                 "if_expression", "match_expression", "for_expression",
                 "while_expression", "block", "tuple_expression",
                 "array_expression", "lambda_expression", "field_expression",
                 "index_expression"]
        for e in exprs:
            self.assertIn(e, self.content, f"Missing expression: {e}")

    def test_has_all_types(self):
        types = ["primitive_type", "qubit_type", "qubits_type", "measured_type",
                 "fn_type", "tuple_type", "array_type", "generic_type"]
        for t in types:
            self.assertIn(t, self.content, f"Missing type: {t}")

    def test_has_all_statements(self):
        stmts = ["let_statement", "return_statement"]
        for s in stmts:
            self.assertIn(s, self.content, f"Missing statement: {s}")

    def test_has_keywords(self):
        for kw in ["'fn'", "'let'", "'mut'", "'if'", "'else'", "'match'",
                   "'for'", "'in'", "'while'", "'return'", "'import'",
                   "'struct'", "'enum'", "'trait'", "'impl'", "'const'",
                   "'pure'", "'quantum'", "'prob'",
                   "'true'", "'false'", "'Qubit'", "'Qubits'", "'Measured'",
                   "'Bool'", "'Int'", "'Float'", "'Complex'", "'Vec'",
                   "'Params'", "'String'", "'Unit'"]:
            self.assertIn(kw, self.content, f"Missing keyword/type token: {kw}")

    def test_has_extras(self):
        self.assertIn("extras:", self.content)
        self.assertIn("$.comment", self.content)
        self.assertIn("$.doc_comment", self.content)

    def test_has_precedence(self):
        self.assertIn("prec.left", self.content)
        self.assertIn("prec", self.content)


class TestTreeSitterPackage(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "package.json")) as f:
            self.pkg = json.load(f)

    def test_name(self):
        self.assertEqual(self.pkg["name"], "tree-sitter-gala")

    def test_has_generate_script(self):
        self.assertIn("generate", self.pkg.get("scripts", {}))

    def test_has_tree_sitter_cli(self):
        self.assertIn("tree-sitter-cli", self.pkg.get("devDependencies", {}))


class TestTreeSitterCargo(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "Cargo.toml")) as f:
            self.content = f.read()

    def test_package_name(self):
        self.assertIn("name = \"tree-sitter-gala\"", self.content)

    def test_tree_sitter_dep(self):
        self.assertIn("tree-sitter", self.content)

    def test_build_dep(self):
        self.assertIn("cc", self.content)


class TestTreeSitterRustBindings(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "bindings/rust/lib.rs")) as f:
            self.content = f.read()

    def test_language_function(self):
        self.assertIn("fn language()", self.content)

    def test_node_kind_names(self):
        self.assertIn("node_kind_names", self.content)
        self.assertIn("source_file", self.content)
        self.assertIn("function_definition", self.content)
        self.assertIn("type", self.content)

    def test_highlights_query(self):
        self.assertIn("highlights_query", self.content)
        self.assertIn("@keyword", self.content)
        self.assertIn("@type.builtin", self.content)


class TestTreeSitterPythonBindings(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "bindings/python/tree_sitter_gala/__init__.py")) as f:
            self.content = f.read()

    def test_language_function(self):
        self.assertIn("def language", self.content)

    def test_create_parser(self):
        self.assertIn("def create_parser", self.content)


class TestTreeSitterNodeBindings(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TS_DIR, "bindings/node/index.js")) as f:
            self.content = f.read()

    def test_load_function(self):
        self.assertIn("function load", self.content)
        self.assertIn("tree-sitter", self.content)


if __name__ == "__main__":
    unittest.main()