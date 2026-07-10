#!/usr/bin/env python3
"""Tests for the Prism.js grammar."""

import os
import re
import unittest

PRISM_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "prism"))


class TestPrism(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(PRISM_DIR, "gala.js")) as f:
            self.content = f.read()

    def test_prism_wrapper(self):
        self.assertIn("Prism.languages.gala", self.content)
        self.assertIn("(function (Prism)", self.content)

    def test_keyword_patterns(self):
        self.assertIn("keyword", self.content)
        self.assertIn("effect", self.content)
        self.assertIn("quantum", self.content)

    def test_builtin_patterns(self):
        self.assertIn("gate", self.content)
        self.assertIn("builtin", self.content)

    def test_type_pattern(self):
        self.assertIn("type", self.content)
        for t in ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float",
                  "Complex", "Vec", "Params", "String", "Unit", "Self"]:
            self.assertIn(t, self.content, f"Missing type: {t}")

    def test_keywords_in_pattern(self):
        keywords = ["fn", "let", "mut", "if", "else", "match", "for", "in",
                   "while", "return", "import", "type", "struct", "enum",
                   "trait", "impl", "const", "where"]
        for kw in keywords:
            self.assertIn(kw, self.content, f"Missing keyword: {kw}")

    def test_effects_in_pattern(self):
        for e in ["pure", "quantum", "prob"]:
            self.assertIn(e, self.content, f"Missing effect: {e}")

    def test_gates_in_pattern(self):
        for g in ["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap"]:
            self.assertIn(g, self.content, f"Missing gate: {g}")

    def test_number_patterns(self):
        self.assertIn("number", self.content)
        self.assertIn("0[xX]", self.content)
        self.assertIn("0[bB]", self.content)

    def test_comments(self):
        self.assertIn("comment", self.content)
        self.assertIn("//", self.content)
        self.assertIn("\\/\\*", self.content)
        self.assertIn("doc-comment", self.content)

    def test_operators(self):
        self.assertIn("operator", self.content)
        self.assertIn("->", self.content)


if __name__ == "__main__":
    unittest.main()