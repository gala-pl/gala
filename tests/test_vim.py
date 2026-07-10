#!/usr/bin/env python3
"""Tests for the Vim/Neovim syntax, indent, ftplugin, and ftdetect files."""

import os
import re
import unittest

VIM_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "vim"))
FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")


class TestVimFileDetection(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VIM_DIR, "ftdetect/gala.vim")) as f:
            self.content = f.read()

    def test_sets_filetype(self):
        self.assertIn("setfiletype gala", self.content)

    def test_matches_gala_extension(self):
        self.assertIn("*.gala", self.content)


class TestVimSyntax(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VIM_DIR, "syntax/gala.vim")) as f:
            self.content = f.read()

    def test_syntax_enabled(self):
        self.assertIn("if exists(\"b:current_syntax\")", self.content)
        self.assertIn("let b:current_syntax = \"gala\"", self.content)

    def test_comment_groups(self):
        self.assertIn("galaLineComment", self.content)
        self.assertIn("galaBlockComment", self.content)
        self.assertIn("galaDocComment", self.content)

    def test_string_highlight(self):
        self.assertIn("galaString", self.content)
        self.assertIn("galaEscape", self.content)

    def test_number_highlight(self):
        self.assertIn("galaInteger", self.content)
        self.assertIn("galaHex", self.content)
        self.assertIn("galaBin", self.content)
        self.assertIn("galaFloat", self.content)
        self.assertIn("galaComplex", self.content)

    def test_type_highlight(self):
        self.assertIn("galaType", self.content)
        for t in ["Qubit", "Qubits", "Measured", "Bool", "Int", "Float",
                  "Complex", "Vec", "Params", "String", "Unit", "Self"]:
            self.assertIn(t, self.content, f"Missing type in syntax: {t}")

    def test_effect_highlight(self):
        self.assertIn("galaEffect", self.content)
        for e in ["pure", "quantum", "prob"]:
            self.assertIn(e, self.content, f"Missing effect: {e}")

    def test_quantum_keywords(self):
        self.assertIn("galaQuantum", self.content)
        for q in ["qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop"]:
            self.assertIn(q, self.content, f"Missing quantum keyword: {q}")

    def test_gate_highlight(self):
        self.assertIn("galaGate", self.content)
        for g in ["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap"]:
            self.assertIn(g, self.content, f"Missing gate: {g}")

    def test_keywords_present(self):
        keywords = ["fn", "let", "mut", "if", "else", "match", "for", "in",
                    "while", "return", "import", "type", "struct", "enum", "trait",
                    "impl", "const", "where"]
        for kw in keywords:
            self.assertIn(kw, self.content, f"Missing keyword: {kw}")

    def test_highlight_links(self):
        links = ["Comment", "SpecialComment", "Type", "Keyword", "Special",
                  "Function", "Boolean", "Operator", "Number", "Float", "String",
                  "Delimiter"]
        for link in links:
            self.assertIn(link, self.content, f"Missing highlight link: {link}")

    def test_keyword_group_exists(self):
        self.assertIn("galaKeyword", self.content)

    def test_function_highlight(self):
        self.assertIn("galaFunction", self.content)

    def test_builtin_highlight(self):
        self.assertIn("galaBuiltin", self.content)


class TestVimIndent(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VIM_DIR, "indent/gala.vim")) as f:
            self.content = f.read()

    def test_autoindent(self):
        self.assertIn("autoindent", self.content)

    def test_shiftwidth(self):
        self.assertIn("shiftwidth=4", self.content)

    def test_expandtab(self):
        self.assertIn("expandtab", self.content)

    def test_cinwords(self):
        self.assertIn("fn", self.content)
        self.assertIn("if", self.content)
        self.assertIn("for", self.content)


class TestVimFtplugin(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VIM_DIR, "ftplugin/gala.vim")) as f:
            self.content = f.read()

    def test_commentstring(self):
        self.assertIn("commentstring=//", self.content)

    def test_makeprg(self):
        self.assertIn("gala", self.content)
        self.assertIn("check", self.content)

    def test_path(self):
        self.assertIn("src/**", self.content)

    def test_suffixesadd(self):
        self.assertIn("suffixesadd=.gala", self.content)


class TestVimAfterSyntax(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(VIM_DIR, "after/syntax/gala.vim")) as f:
            self.content = f.read()

    def test_error_code(self):
        self.assertIn("galaErrorCode", self.content)
        self.assertIn("E", self.content)

    def test_doc_tag(self):
        self.assertIn("galaDocTag", self.content)
        self.assertIn("@", self.content)


if __name__ == "__main__":
    unittest.main()