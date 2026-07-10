package com.gala.lang

import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.options.colors.AttributesDescriptor
import com.intellij.openapi.options.colors.ColorDescriptor
import com.intellij.openapi.options.colors.ColorSettingsPage
import javax.swing.Icon

class GalaColorSettingsPage : ColorSettingsPage {
    override fun getIcon(): Icon? = null
    override fun getHighlighter(): SyntaxHighlighter = GalaSyntaxHighlighter()

    override fun getDemoText() = """
/// Bell pair circuit
fn bell() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    let (a, b) = cx(a, b)
    (a, b)
}

/// Fair coin — crosses the measurement boundary
fn coin() -> Measured<Bool> prob {
    let q = qubit()
    h(q)
    measure(q)
}

/// Variational classifier
fn classify(x: Vec<Float>, θ: Params) -> Measured<Bool> prob {
    let q = ansatz(encode(x), θ)
    measure(q[0])
}
""".trimIndent()

    override fun getAdditionalHighlightingTagToDescriptorMap(): Map<String, TextAttributesKey>? = null

    override fun getAttributeDescriptors() = arrayOf(
        AttributesDescriptor("Keyword", GalaHighlightTokens.KEYWORD),
        AttributesDescriptor("Declaration keyword", GalaHighlightTokens.DECLARATION),
        AttributesDescriptor("Type", GalaHighlightTokens.TYPE),
        AttributesDescriptor("Effect", GalaHighlightTokens.EFFECT),
        AttributesDescriptor("Quantum operation", GalaHighlightTokens.QUANTUM),
        AttributesDescriptor("Gate", GalaHighlightTokens.GATE),
        AttributesDescriptor("Built-in", GalaHighlightTokens.BUILTIN),
        AttributesDescriptor("String", GalaHighlightTokens.STRING),
        AttributesDescriptor("Number", GalaHighlightTokens.NUMBER),
        AttributesDescriptor("Complex number", GalaHighlightTokens.COMPLEX),
        AttributesDescriptor("Line comment", GalaHighlightTokens.COMMENT),
        AttributesDescriptor("Block comment", GalaHighlightTokens.BLOCK_COMMENT),
        AttributesDescriptor("Doc comment", GalaHighlightTokens.DOC_COMMENT),
        AttributesDescriptor("Operator", GalaHighlightTokens.OPERATOR),
        AttributesDescriptor("Parentheses", GalaHighlightTokens.PAREN),
        AttributesDescriptor("Braces", GalaHighlightTokens.BRACE),
        AttributesDescriptor("Brackets", GalaHighlightTokens.BRACKET)
    )

    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY
}

object GalaHighlightTokens {
    val KEYWORD = GalaHighlightingKeys.KEYWORD
    val DECLARATION = GalaHighlightingKeys.DECLARATION
    val TYPE = GalaHighlightingKeys.TYPE
    val EFFECT = GalaHighlightingKeys.EFFECT
    val QUANTUM = GalaHighlightingKeys.QUANTUM
    val GATE = GalaHighlightingKeys.GATE
    val BUILTIN = GalaHighlightingKeys.BUILTIN
    val STRING = GalaHighlightingKeys.STRING
    val NUMBER = GalaHighlightingKeys.NUMBER
    val COMPLEX = GalaHighlightingKeys.COMPLEX
    val COMMENT = GalaHighlightingKeys.LINE_COMMENT
    val BLOCK_COMMENT = GalaHighlightingKeys.BLOCK_COMMENT
    val DOC_COMMENT = GalaHighlightingKeys.DOC_COMMENT
    val OPERATOR = GalaHighlightingKeys.OPERATOR
    val PAREN = GalaHighlightingKeys.PAREN
    val BRACE = GalaHighlightingKeys.BRACE
    val BRACKET = GalaHighlightingKeys.BRACKET
    val COMMA = GalaHighlightingKeys.COMMA
    val SEMICOLON = GalaHighlightingKeys.SEMICOLON
    val DOT = GalaHighlightingKeys.DOT
    val BAD_CHAR = GalaHighlightingKeys.BAD_CHAR
}