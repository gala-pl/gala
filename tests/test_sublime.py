#!/usr/bin/env python3
"""Tests for the Sublime Text syntax and completions."""

import os
import yaml
import json
import unittest

SUBLIME_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "sublime"))


class TestSublimeSyntax(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(SUBLIME_DIR, "Gala.sublime-syntax")) as f:
            self.syntax = yaml.safe_load(f)

    def test_name(self):
        self.assertEqual(self.syntax["name"], "Gala")

    def test_file_extensions(self):
        self.assertIn("gala", self.syntax["file_extensions"])

    def test_scope(self):
        self.assertEqual(self.syntax["scope"], "source.gala")

    def test_has_contexts(self):
        required = ["main", "comments", "strings", "numbers", "keywords",
                    "types", "effects", "quantum-keywords", "operators"]
        for ctx in required:
            self.assertIn(ctx, self.syntax["contexts"], f"Missing context: {ctx}")

    def test_keywords_all_present(self):
        context = self.syntax["contexts"]["keywords"]
        matches = []
        for p in context:
            if "match" in p:
                matches.append(p["match"])
        full = " ".join(matches)
        for kw in ["fn", "let", "if", "else", "match", "for", "in", "while", "return",
                   "import", "type", "struct", "enum", "trait", "impl", "const", "where",
                   "true", "false"]:
            self.assertIn(kw, full, f"Missing keyword: {kw}")

    def test_types_all_present(self):
        context = self.syntax["contexts"]["types"]
        matches = [p["match"] for p in context if "match" in p]
        full = " ".join(matches)
        for t in ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float", "Complex",
                  "Vec", "Params", "String", "Unit", "Self"]:
            self.assertIn(t, full, f"Missing type: {t}")


class TestSublimeCompletions(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(SUBLIME_DIR, "Gala.sublime-completions")) as f:
            self.completions = json.load(f)

    def test_scope(self):
        self.assertEqual(self.completions["scope"], "source.gala")

    def test_has_completions(self):
        triggers = [c["trigger"] for c in self.completions["completions"]
                    if isinstance(c, dict)]
        names = {t.split("\t")[0] for t in triggers}
        for name in ["fn", "let", "letm", "if", "ife", "match", "for", "return",
                     "struct", "enum", "trait", "impl", "import", "qfn", "pfn",
                     "bell", "measure", "grad", "adj", "ctrl"]:
            self.assertIn(name, names, f"Missing completion trigger: {name}")


if __name__ == "__main__":
    unittest.main()