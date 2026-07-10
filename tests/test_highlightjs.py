#!/usr/bin/env python3
"""Tests for the highlight.js grammar."""

import os
import re
import unittest

HIGHLIGHT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "highlight-js"))


class TestHighlightJS(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(HIGHLIGHT_DIR, "gala.js")) as f:
            self.content = f.read()

    def test_exports_function(self):
        self.assertIn("export default function", self.content)
        self.assertIn("hljs.registerLanguage", self.content) if "registerLanguage" in self.content else None

    def test_keywords_section(self):
        # Check keyword groups
        self.assertIn("keyword:", self.content)
        self.assertIn("type:", self.content)
        self.assertIn("effect:", self.content)
        self.assertIn("quantum:", self.content)
        self.assertIn("literal:", self.content)
        self.assertIn("built_in:", self.content)

    def test_all_keywords_present(self):
        keywords = ["fn", "let", "mut", "if", "else", "match", "for", "in",
                   "while", "return", "import", "type", "struct", "enum",
                   "trait", "impl", "const", "where"]
        for kw in keywords:
            self.assertIn(f"'{kw}'", self.content, f"Missing keyword: {kw}")

    def test_all_types_present(self):
        types = ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float",
                  "Complex", "Vec", "Params", "String", "Unit", "Self"]
        for t in types:
            self.assertIn(f"'{t}'", self.content, f"Missing type: {t}")

    def test_all_effects_present(self):
        for e in ["pure", "quantum", "prob"]:
            self.assertIn(f"'{e}'", self.content, f"Missing effect: {e}")

    def test_all_quantum_keywords_present(self):
        for q in ["qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop"]:
            self.assertIn(f"'{q}'", self.content, f"Missing quantum keyword: {q}")

    def test_all_gates_present(self):
        for g in ["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap"]:
            self.assertIn(f"'{g}'", self.content, f"Missing gate: {g}")

    def test_has_comments(self):
        self.assertIn("comment", self.content)
        self.assertIn("//", self.content)
        self.assertIn("/*", self.content)
        self.assertIn("*/", self.content)

    def test_has_numbers(self):
        self.assertIn("number", self.content)

    def test_has_strings(self):
        self.assertIn("string", self.content)
        self.assertIn("escape", self.content)

    def test_has_operators(self):
        self.assertIn("operator", self.content)

    def test_has_function_pattern(self):
        self.assertIn("function", self.content)
        self.assertIn("beginKeywords", self.content)


if __name__ == "__main__":
    unittest.main()