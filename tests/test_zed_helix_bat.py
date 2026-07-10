#!/usr/bin/env python3
"""Tests for Zed, Helix, and bat editor extensions."""

import os
import yaml
import unittest
import re

EXTENSIONS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions"))

ZED_DIR = os.path.join(EXTENSIONS_DIR, "zed")
HELIX_DIR = os.path.join(EXTENSIONS_DIR, "helix")
BAT_DIR = os.path.join(EXTENSIONS_DIR, "bat")


class TestZedHighlights(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(ZED_DIR, "gala.scm")) as f:
            self.content = f.read()

    def test_has_comments(self):
        self.assertIn("comment", self.content)

    def test_has_keywords(self):
        self.assertIn("@keyword", self.content)

    def test_has_types(self):
        self.assertIn("@type", self.content)

    def test_has_quantum_types(self):
        self.assertIn("@type.quantum", self.content)

    def test_has_functions(self):
        self.assertIn("@function", self.content)

    def test_has_operators(self):
        self.assertIn("@operator", self.content)

    def test_has_punctuation(self):
        self.assertIn("@punctuation", self.content)

    def test_keywords_mentioned(self):
        for kw in ["fn", "let", "if", "else", "match", "for", "while", "return",
                   "import", "struct", "enum", "trait", "impl"]:
            self.assertIn(kw, self.content, f"Missing keyword: {kw}")

    def test_types_mentioned(self):
        for t in ["primitive_type", "qubit_type", "qubits_type", "measured_type"]:
            self.assertIn(t, self.content, f"Missing type node: {t}")

    def test_effects_mentioned(self):
        for e in ["pure", "quantum", "prob"]:
            self.assertIn(e, self.content, f"Missing effect: {e}")

    def test_quantum_ops_mentioned(self):
        for q in ["qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop"]:
            self.assertIn(q, self.content, f"Missing quantum op: {q}")


class TestHelixLanguagesConfig(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(HELIX_DIR, "languages.toml")) as f:
            self.content = f.read()

    def test_language_name(self):
        self.assertIn('name = "gala"', self.content)

    def test_scope(self):
        self.assertIn('scope = "source.gala"', self.content)

    def test_file_types(self):
        self.assertIn('file-types = ["gala"]', self.content)

    def test_comment_token(self):
        self.assertIn('comment-token = "//"', self.content)

    def test_block_comment(self):
        self.assertIn("block-comment-tokens", self.content)
        self.assertIn("/*", self.content)
        self.assertIn("*/", self.content)

    def test_language_server(self):
        self.assertIn("gala-lsp", self.content)

    def test_auto_format(self):
        self.assertIn("auto-format = true", self.content)

    def test_formatter(self):
        self.assertIn('formatter = { command = "gala"', self.content)

    def test_indent_config(self):
        self.assertIn("tab-width = 4", self.content)
        self.assertIn("unit = \"    \"", self.content)

    def test_grammar_reference(self):
        self.assertIn("[[grammar]]", self.content)
        self.assertIn('name = "gala"', self.content)
        self.assertIn("tree-sitter-gala", self.content)


class TestHelixHighlights(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(HELIX_DIR, "queries/gala/highlights.scm")) as f:
            self.content = f.read()

    def test_comment_capture(self):
        self.assertIn("@comment", self.content)

    def test_keyword_capture(self):
        self.assertIn("@keyword", self.content)

    def test_type_capture(self):
        self.assertIn("@type", self.content)

    def test_function_capture(self):
        self.assertIn("@function", self.content)

    def test_string_capture(self):
        self.assertIn("@string", self.content)

    def test_number_capture(self):
        self.assertIn("@number", self.content)

    def test_operator_capture(self):
        self.assertIn("@operator", self.content)

    def test_keywords_mentioned(self):
        keywords = ["fn", "let", "if", "else", "match", "for", "while", "return",
                    "import", "struct", "enum", "trait", "impl", "const"]
        for kw in keywords:
            self.assertIn(kw, self.content, f"Missing keyword: {kw}")


class TestHelixIndents(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(HELIX_DIR, "queries/gala/indents.scm")) as f:
            self.content = f.read()

    def test_has_block_indent(self):
        self.assertIn("@indent", self.content)

    def test_has_block_outdent(self):
        self.assertIn("@outdent", self.content)

    def test_function_indent(self):
        self.assertIn("function_definition", self.content)
        self.assertIn("@indent", self.content)

    def test_if_indent(self):
        self.assertIn("if_expression", self.content)


class TestHelixFolds(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(HELIX_DIR, "queries/gala/folds.scm")) as f:
            self.content = f.read()

    def test_has_folds(self):
        self.assertIn("@fold", self.content)

    def test_block_fold(self):
        self.assertIn("block", self.content)

    def test_comment_fold(self):
        self.assertIn("comment", self.content)

    def test_function_fold(self):
        self.assertIn("function_definition", self.content)

    def test_struct_fold(self):
        self.assertIn("struct_definition", self.content)

    def test_match_fold(self):
        self.assertIn("match_expression", self.content)


class TestBatSyntax(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(BAT_DIR, "Gala.sublime-syntax")) as f:
            self.syntax = yaml.safe_load(f)

    def test_name(self):
        self.assertEqual(self.syntax["name"], "Gala")

    def test_file_extensions(self):
        self.assertIn("gala", self.syntax["file_extensions"])

    def test_scope(self):
        self.assertEqual(self.syntax["scope"], "source.gala")

    def test_has_keywords_context(self):
        self.assertIn("keywords", self.syntax["contexts"])

    def test_keywords_mentioned(self):
        context = self.syntax["contexts"]["keywords"]
        matches = [p.get("match", "") for p in context]
        full = " ".join(matches)
        for kw in ["fn", "let", "if", "else", "match", "for", "in", "while", "return",
                   "import", "type", "struct", "enum", "trait", "impl", "const", "where",
                   "true", "false"]:
            self.assertIn(kw, full, f"Missing keyword in bat syntax: {kw}")

    def test_types_mentioned(self):
        context = self.syntax["contexts"]["types"]
        matches = [p.get("match", "") for p in context]
        full = " ".join(matches)
        for t in ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float",
                  "Complex", "Vec", "Params", "String", "Unit", "Self"]:
            self.assertIn(t, full, f"Missing type in bat syntax: {t}")


if __name__ == "__main__":
    unittest.main()