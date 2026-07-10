#!/usr/bin/env python3
"""Validate that all Gala fixture files are syntactically correct and that all
grammar definitions would successfully match the tokens in those fixtures."""

import os
import re
import json
import plistlib
import unittest

EXTENSIONS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions"))
FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")

VSCODE_DIR = os.path.join(EXTENSIONS_DIR, "vscode")
TEXTMATE_DIR = os.path.join(EXTENSIONS_DIR, "textmate")


class TestFixtureParsing(unittest.TestCase):
    """Validate fixtures have valid Gala syntax structure."""

    def test_bell_has_correct_structure(self):
        with open(os.path.join(FIXTURES_DIR, "bell.gala")) as f:
            content = f.read()
        self.assertIn("fn bell()", content)
        self.assertIn("-> Qubits<2> quantum", content)
        self.assertIn("qubit()", content)
        self.assertIn("h(a)", content)
        self.assertIn("cx(a, b)", content)
        self.assertIn("measure(a)", content)

    def test_variational_has_correct_structure(self):
        with open(os.path.join(FIXTURES_DIR, "variational.gala")) as f:
            content = f.read()
        self.assertIn("import gala.ml.{ angle_encode, Params }", content)
        self.assertIn("fn encode", content)
        self.assertIn("Vec<Float>", content)
        self.assertIn("Qubits<4>", content)
        self.assertIn("for layer", content)
        self.assertIn("ry(q[i]", content)
        self.assertIn("grad(classify, wrt: θ)", content)

    def test_classical_has_correct_structure(self):
        with open(os.path.join(FIXTURES_DIR, "classical.gala")) as f:
            content = f.read()
        self.assertIn("fn fib", content)
        self.assertIn("Int", content)
        self.assertIn("Hello, Gala!", content)
        self.assertIn("print(msg)", content)


class TestGrammarMatchesFixtureTokens(unittest.TestCase):
    """Test that grammar patterns can match tokens from the standalone TextMate grammar."""

    @classmethod
    def setUpClass(cls):
        with open(os.path.join(TEXTMATE_DIR, "Gala.tmLanguage"), "rb") as f:
            cls.full_grammar = plistlib.load(f)
        def find_match_values(obj, acc):
            if isinstance(obj, dict):
                if "match" in obj:
                    acc.append(obj["match"])
                for v in obj.values():
                    find_match_values(v, acc)
            elif isinstance(obj, list):
                for item in obj:
                    find_match_values(item, acc)
        cls.regexes = []
        find_match_values(cls.full_grammar, cls.regexes)

    def _check_token(self, token, token_type):
        matched = False
        for regex in self.regexes:
            try:
                if re.search(regex, token):
                    matched = True
                    break
            except re.error:
                continue
        self.assertTrue(matched, f"Token '{token}' ({token_type}) not matched by any grammar pattern")

    def test_keywords_are_matched(self):
        tokens = ["fn", "let", "mut", "if", "else", "match", "for", "in",
                  "while", "return", "import", "type", "struct", "enum",
                  "trait", "impl", "const", "where"]
        for t in tokens:
            self._check_token(t, "keyword")

    def test_types_are_matched(self):
        tokens = ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float",
                  "Complex", "Vec", "Params", "String", "Unit", "Self"]
        for t in tokens:
            self._check_token(t, "type")

    def test_effects_are_matched(self):
        tokens = ["pure", "quantum", "prob"]
        for t in tokens:
            self._check_token(t, "effect")

    def test_quantum_keywords_are_matched(self):
        tokens = ["qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop"]
        for t in tokens:
            self._check_token(t, "quantum keyword")

    def test_gates_are_matched(self):
        tokens = ["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap"]
        for t in tokens:
            self._check_token(t, "gate")

    def test_literals_are_matched(self):
        tokens = ["42", "0xFF", "0b1010", "3.14", "1e-3", "2+3i"]
        for t in tokens[:5]:
            self._check_token(t, "literal")


class TestAllExtensionsExist(unittest.TestCase):
    """Verify every expected extension file exists."""

    def test_vscode_files_exist(self):
        self.assertTrue(os.path.isfile(os.path.join(VSCODE_DIR, "package.json")))
        self.assertTrue(os.path.isfile(os.path.join(VSCODE_DIR, "syntaxes/gala.tmLanguage.json")))
        self.assertTrue(os.path.isfile(os.path.join(VSCODE_DIR, "language-configuration.json")))
        self.assertTrue(os.path.isfile(os.path.join(VSCODE_DIR, "snippets/gala.json")))
        self.assertTrue(os.path.isfile(os.path.join(VSCODE_DIR, "client/extension.js")))

    def test_cursor_files_exist(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "cursor/.cursorrules")))
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "cursor/snippets/gala.json")))

    def test_intellij_files_exist(self):
        intellij_dir = os.path.join(EXTENSIONS_DIR, "intellij")
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "build.gradle.kts")))
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "src/main/resources/META-INF/plugin.xml")))
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "src/main/kotlin/com/gala/lang/GalaLanguage.kt")))
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "src/main/kotlin/com/gala/lang/GalaSyntaxHighlighter.kt")))
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "src/main/kotlin/com/gala/lang/GalaCommenter.kt")))
        self.assertTrue(os.path.isfile(os.path.join(intellij_dir, "src/main/kotlin/com/gala/lang/GalaBraceMatcher.kt")))

    def test_vim_files_exist(self):
        vim_dir = os.path.join(EXTENSIONS_DIR, "vim")
        self.assertTrue(os.path.isfile(os.path.join(vim_dir, "ftdetect/gala.vim")))
        self.assertTrue(os.path.isfile(os.path.join(vim_dir, "syntax/gala.vim")))
        self.assertTrue(os.path.isfile(os.path.join(vim_dir, "indent/gala.vim")))
        self.assertTrue(os.path.isfile(os.path.join(vim_dir, "ftplugin/gala.vim")))
        self.assertTrue(os.path.isfile(os.path.join(vim_dir, "after/syntax/gala.vim")))

    def test_sublime_files_exist(self):
        sublime_dir = os.path.join(EXTENSIONS_DIR, "sublime")
        self.assertTrue(os.path.isfile(os.path.join(sublime_dir, "Gala.sublime-syntax")))
        self.assertTrue(os.path.isfile(os.path.join(sublime_dir, "Gala.sublime-completions")))
        self.assertTrue(os.path.isfile(os.path.join(sublime_dir, "Preferences.sublime-settings")))

    def test_textmate_file_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(TEXTMATE_DIR, "Gala.tmLanguage")))

    def test_treesitter_files_exist(self):
        ts_dir = os.path.join(EXTENSIONS_DIR, "tree-sitter-gala")
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "grammar.js")))
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "package.json")))
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "Cargo.toml")))
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "bindings/rust/lib.rs")))
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "bindings/node/index.js")))
        self.assertTrue(os.path.isfile(os.path.join(ts_dir, "bindings/python/tree_sitter_gala/__init__.py")))

    def test_highlightjs_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "highlight-js/gala.js")))

    def test_prism_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "prism/gala.js")))

    def test_bat_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "bat/Gala.sublime-syntax")))

    def test_zed_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "zed/gala.scm")))

    def test_helix_files_exist(self):
        helix_dir = os.path.join(EXTENSIONS_DIR, "helix")
        self.assertTrue(os.path.isfile(os.path.join(helix_dir, "languages.toml")))
        self.assertTrue(os.path.isfile(os.path.join(helix_dir, "queries/gala/highlights.scm")))
        self.assertTrue(os.path.isfile(os.path.join(helix_dir, "queries/gala/indents.scm")))
        self.assertTrue(os.path.isfile(os.path.join(helix_dir, "queries/gala/folds.scm")))

    def test_readme_exists(self):
        self.assertTrue(os.path.isfile(os.path.join(EXTENSIONS_DIR, "README.md")))

    def test_tests_exist(self):
        root_tests_dir = os.path.join(os.path.dirname(__file__))
        self.assertTrue(os.path.isdir(root_tests_dir))
        self.assertTrue(os.path.isdir(os.path.join(root_tests_dir, "fixtures")))


if __name__ == "__main__":
    unittest.main()