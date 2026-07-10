#!/usr/bin/env python3
"""Tests for Cursor and standalone TextMate grammar."""

import os
import json
import plistlib
import unittest

EXTENSIONS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions"))
CURSOR_DIR = os.path.join(EXTENSIONS_DIR, "cursor")
TEXTMATE_DIR = os.path.join(EXTENSIONS_DIR, "textmate")


class TestCursorRules(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(CURSOR_DIR, ".cursorrules")) as f:
            self.content = f.read()

    def test_mentions_gala(self):
        self.assertIn("Gala", self.content)

    def test_mentions_file_extension(self):
        self.assertIn(".gala", self.content)

    def test_mentions_effects(self):
        self.assertIn("pure", self.content)
        self.assertIn("quantum", self.content)
        self.assertIn("prob", self.content)

    def test_mentions_linear_types(self):
        self.assertIn("linear", self.content) or self.assertIn("Linear", self.content)
        self.assertIn("Qubit", self.content)

    def test_mentions_gates(self):
        for g in ["h", "x", "y", "z", "cx", "rx", "ry", "rz"]:
            self.assertIn(g, self.content, f"Missing gate: {g}")

    def test_mentions_adjoint_grad(self):
        self.assertIn("adjoint", self.content)
        self.assertIn("grad", self.content)

    def test_mentions_toolchain(self):
        for cmd in ["check", "run", "build", "test", "fmt", "repl", "explain", "lsp"]:
            self.assertIn(cmd, self.content, f"Missing command reference: {cmd}")

    def test_has_code_example(self):
        self.assertIn("fn bell", self.content)
        self.assertIn("cx(a, b)", self.content)

    def test_has_project_structure(self):
        self.assertIn("gala.toml", self.content)
        self.assertIn("main.gala", self.content)


class TestCursorSnippets(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(CURSOR_DIR, "snippets/gala.json")) as f:
            self.snippets = json.load(f)

    def test_has_critical_snippets(self):
        names = list(self.snippets.keys())
        for name in ["Function definition", "Quantum function", "Probabilistic function",
                     "Pure function", "Bell pair", "Let binding", "If expression",
                     "Match expression", "For loop", "Measure qubit", "Gradient",
                     "Adjoint", "Controlled", "Struct definition", "Import",
                     "Main function"]:
            self.assertIn(name, names, f"Missing snippet: {name}")


class TestStandaloneTextMate(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(TEXTMATE_DIR, "Gala.tmLanguage"), "rb") as f:
            self.grammar = plistlib.load(f)

    def _get_value(self, key):
        val = self.grammar.get(key)
        self.assertIsNotNone(val, f"Missing key '{key}'")
        return val

    def test_name(self):
        self.assertEqual(self._get_value("name"), "Gala")

    def test_scope_name(self):
        self.assertEqual(self._get_value("scopeName"), "source.gala")

    def test_file_types(self):
        types = self._get_value("fileTypes")
        self.assertIn("gala", types)

    def test_repository_keys(self):
        repo = self._get_value("repository")
        required = ["comments", "doc-comments", "strings", "numbers",
                    "keywords", "types", "effects", "quantum-keywords",
                    "gates", "builtins", "operators", "punctuation", "identifiers"]
        for k in required:
            self.assertIn(k, repo, f"Missing repository key: {k}")

    def test_patterns(self):
        patterns = self._get_value("patterns")
        self.assertTrue(len(patterns) > 0)
        first = patterns[0]
        self.assertIn("include", first)


if __name__ == "__main__":
    unittest.main()