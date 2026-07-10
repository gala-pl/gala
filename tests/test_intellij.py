#!/usr/bin/env python3
"""Tests for the IntelliJ IDEA plugin."""

import os
import xml.etree.ElementTree as ET
import unittest

INTELLIJ_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "extensions", "intellij"))


class TestIntelliJPluginXml(unittest.TestCase):
    def setUp(self):
        tree = ET.parse(os.path.join(INTELLIJ_DIR, "src/main/resources/META-INF/plugin.xml"))
        self.root = tree.getroot()

    def test_plugin_id(self):
        self.assertEqual(self.root.find("id").text, "com.gala.lang")

    def test_plugin_name(self):
        self.assertEqual(self.root.find("name").text, "Gala Language Support")

    def test_has_language_extensions(self):
        extensions = self.root.find(".//extensions")
        self.assertIsNotNone(extensions)
        kids = list(extensions)
        self.assertTrue(len(kids) > 0)

    def test_has_file_type(self):
        ext = self.root.findall(".//fileType")
        self.assertTrue(len(ext) >= 1)
        self.assertEqual(ext[0].get("extensions"), "gala")
        self.assertEqual(ext[0].get("name"), "Gala")

    def test_has_syntax_highlighter(self):
        hl = self.root.findall(".//lang.syntaxHighlighterFactory")
        self.assertTrue(len(hl) >= 1)
        self.assertEqual(hl[0].get("language"), "gala")

    def test_has_commenter(self):
        cm = self.root.findall(".//lang.commenter")
        self.assertTrue(len(cm) >= 1)
        self.assertEqual(cm[0].get("language"), "gala")

    def test_has_brace_matcher(self):
        bm = self.root.findall(".//lang.braceMatcher")
        self.assertTrue(len(bm) >= 1)
        self.assertEqual(bm[0].get("language"), "gala")


class TestIntelliJSyntaxHighlighter(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(INTELLIJ_DIR, "src/main/kotlin/com/gala/lang",
                               "GalaSyntaxHighlighter.kt")) as f:
            self.content = f.read()

    def test_has_gala_token_types(self):
        required = ["KW_FN", "KW_LET", "KW_IF", "KW_ELSE", "KW_MATCH",
                    "KW_FOR", "KW_IN", "KW_WHILE", "KW_RETURN", "KW_IMPORT",
                    "KW_STRUCT", "KW_ENUM", "KW_TRAIT", "KW_IMPL", "KW_CONST",
                    "EFFECT_PURE", "EFFECT_QUANTUM", "EFFECT_PROB",
                    "TYPE_QUBIT", "TYPE_QUBITS", "TYPE_MEASURED",
                    "TYPE_BOOL", "TYPE_INT", "TYPE_FLOAT", "TYPE_COMPLEX",
                    "KW_QUBIT", "KW_MEASURE", "KW_REVERSE", "KW_ADJOINT",
                    "KW_CONTROL", "KW_GRAD", "KW_DROP",
                    "GATE_H", "GATE_X", "GATE_RX", "GATE_CX", "GATE_SWAP"]
        for name in required:
            self.assertIn(f"val {name}", self.content, f"Missing: {name}")

    def test_has_keywords_set(self):
        self.assertIn("KEYWORDS", self.content)
        self.assertIn("\"fn\"", self.content)
        self.assertIn("\"let\"", self.content)
        self.assertIn("\"if\"", self.content)

    def test_has_types_set(self):
        self.assertIn("TYPES", self.content)
        self.assertIn("\"Qubit\"", self.content)
        self.assertIn("\"Measured\"", self.content)

    def test_has_gates_set(self):
        self.assertIn("GATES", self.content)
        self.assertIn("\"h\"", self.content)
        self.assertIn("\"cx\"", self.content)


class TestIntelliJCommenter(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(INTELLIJ_DIR, "src/main/kotlin/com/gala/lang",
                               "GalaCommenter.kt")) as f:
            self.content = f.read()

    def test_comments(self):
        self.assertIn("getLineCommentPrefix", self.content)
        self.assertIn("getBlockCommentPrefix", self.content)
        self.assertIn("getBlockCommentSuffix", self.content)


class TestIntelliJLanguage(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(INTELLIJ_DIR, "src/main/kotlin/com/gala/lang",
                               "GalaLanguage.kt")) as f:
            self.content = f.read()

    def test_is_sensitive(self):
        self.assertIn("isCaseSensitive", self.content)

    def test_extension(self):
        self.assertIn("getDefaultExtension", self.content)
        self.assertIn("gala", self.content)


class TestIntelliJColorSettings(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(INTELLIJ_DIR, "src/main/kotlin/com/gala/lang",
                               "GalaColorSettingsPage.kt")) as f:
            self.content = f.read()

    def test_demo_text(self):
        self.assertIn("bell()", self.content)
        self.assertIn("classify", self.content)
        self.assertIn("quantum", self.content)
        self.assertIn("prob", self.content)

    def test_attribute_descriptors(self):
        expected = ["Keyword", "Type", "Effect", "Quantum operation",
                     "Gate", "String", "Number", "Line comment", "Operator"]
        for attr in expected:
            self.assertIn(attr, self.content, f"Missing descriptor: {attr}")


if __name__ == "__main__":
    unittest.main()