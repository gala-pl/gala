#!/usr/bin/env python3
"""Tests for the VS Code extension (TextMate grammar, snippets, package.json)."""

import json
import os
import re
import unittest

EXTENSIONS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions"))
VSCODE_DIR = os.path.join(EXTENSIONS_DIR, "vscode")

class TestVSCodePackage(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VSCODE_DIR, "package.json")) as f:
            self.pkg = json.load(f)

    def test_extension_name(self):
        self.assertEqual(self.pkg["name"], "gala")

    def test_language_id(self):
        langs = self.pkg["contributes"]["languages"]
        self.assertTrue(any(l["id"] == "gala" for l in langs))

    def test_gala_file_extension(self):
        langs = self.pkg["contributes"]["languages"]
        for l in langs:
            if l["id"] == "gala":
                self.assertIn(".gala", l["extensions"])
                return
        self.fail("No gala language entry")

    def test_grammar_registered(self):
        grammars = self.pkg["contributes"]["grammars"]
        self.assertTrue(any(g["language"] == "gala" for g in grammars))

    def test_snippets_registered(self):
        snippets = self.pkg["contributes"]["snippets"]
        self.assertTrue(any(s["language"] == "gala" for s in snippets))

    def test_commands(self):
        commands = [c["command"] for c in self.pkg["contributes"]["commands"]]
        expected = ["gala.check", "gala.run", "gala.explain", "gala.showCircuit"]
        for cmd in expected:
            self.assertIn(cmd, commands)

    def test_compiler_path_config(self):
        props = self.pkg["contributes"]["configuration"]["properties"]
        self.assertIn("gala.compilerPath", props)


class TestTextMateGrammar(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VSCODE_DIR, "syntaxes/gala.tmLanguage.json")) as f:
            self.grammar = json.load(f)

    def test_scope_name(self):
        self.assertEqual(self.grammar["scopeName"], "source.gala")

    def test_file_types(self):
        self.assertIn("gala", self.grammar["fileTypes"])

    def test_repository_keys(self):
        expected_keys = {"comments", "doc-comments", "strings", "numbers", "keywords",
                         "types", "effects", "quantum-keywords", "gates", "builtin-functions",
                         "operators", "function-declaration", "lambda", "identifiers",
                         "punctuation"}
        self.assertEqual(set(self.grammar["repository"].keys()), expected_keys)

    def test_keyword_patterns(self):
        keywords = self.grammar["repository"]["keywords"]["patterns"]
        kw_names = [p.get("match", "") for p in keywords]
        full_pattern = " ".join(kw_names)
        for kw in ["fn", "let", "if", "else", "match", "for", "in", "while", "return",
                   "import", "type", "struct", "enum", "trait", "impl", "const", "where",
                   "true", "false"]:
            self.assertIn(kw, full_pattern, f"Missing keyword: {kw}")

    def test_type_patterns(self):
        types = self.grammar["repository"]["types"]["patterns"]
        all_patterns = " ".join([p.get("match", "") for p in types])
        for t in ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float", "Complex",
                  "Vec", "Params", "String", "Unit", "Self"]:
            self.assertIn(t, all_patterns, f"Missing type: {t}")

    def test_effect_patterns(self):
        effects = self.grammar["repository"]["effects"]["patterns"]
        all_patterns = " ".join([p.get("match", "") for p in effects])
        for e in ["pure", "quantum", "prob"]:
            self.assertIn(e, all_patterns, f"Missing effect: {e}")

    def test_quantum_keyword_patterns(self):
        qk = self.grammar["repository"]["quantum-keywords"]["patterns"]
        all_patterns = " ".join([p.get("match", "") for p in qk])
        for k in ["qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop"]:
            self.assertIn(k, all_patterns, f"Missing quantum keyword: {k}")

    def test_gate_patterns(self):
        gates = self.grammar["repository"]["gates"]["patterns"]
        all_patterns = " ".join([p.get("match", "") for p in gates])
        for g in ["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap"]:
            self.assertIn(g, all_patterns, f"Missing gate: {g}")


class TestLanguageConfiguration(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VSCODE_DIR, "language-configuration.json")) as f:
            self.config = json.load(f)

    def test_comments(self):
        self.assertEqual(self.config["comments"]["lineComment"], "//")
        self.assertEqual(self.config["comments"]["blockComment"], ["/*", "*/"])

    def test_brackets(self):
        self.assertIn(["{", "}"], self.config["brackets"])
        self.assertIn(["(", ")"], self.config["brackets"])
        self.assertIn(["[", "]"], self.config["brackets"])

    def test_indentation_rules(self):
        self.assertIn("increaseIndentPattern", self.config["indentationRules"])
        self.assertIn("decreaseIndentPattern", self.config["indentationRules"])


class TestSnippets(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VSCODE_DIR, "snippets/gala.json")) as f:
            self.snippets = json.load(f)

    def test_has_critical_snippets(self):
        critical = ["Function definition", "Bell pair", "Quantum function",
                     "Probabilistic function", "Measure qubit", "Gradient",
                     "Adjoint / reverse", "Controlled operation", "Let binding",
                     "If expression", "For loop", "Match expression", "Struct definition",
                     "Import statement", "Import specific items", "QFT circuit",
                     "Variational ansatz", "Main function"]
        for name in critical:
            self.assertIn(name, self.snippets, f"Missing snippet: {name}")

    def test_snippet_structure(self):
        for name, snippet in self.snippets.items():
            self.assertIn("prefix", snippet, f"Snippet {name} missing prefix")
            self.assertIn("body", snippet, f"Snippet {name} missing body")
            self.assertIn("description", snippet, f"Snippet {name} missing description")

    def test_bell_snippet(self):
        bell = self.snippets["Bell pair"]
        body = "\n".join(bell["body"])
        self.assertIn("fn bell()", body)
        self.assertIn("cx(a, b)", body)


if __name__ == "__main__":
    unittest.main()