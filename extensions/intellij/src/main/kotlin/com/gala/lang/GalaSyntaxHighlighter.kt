package com.gala.lang

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors as D
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.tree.IElementType

class GalaSyntaxHighlighterFactory : SyntaxHighlighterFactory() {
    override fun getSyntaxHighlighter(project: Project?, virtualFile: VirtualFile?) = GalaSyntaxHighlighter()
}

object GalaHighlightingKeys {
    val KEYWORD = TextAttributesKey.createTextAttributesKey("GALA_KEYWORD", D.KEYWORD)
    val DECLARATION = TextAttributesKey.createTextAttributesKey("GALA_DECLARATION", D.KEYWORD)
    val TYPE = TextAttributesKey.createTextAttributesKey("GALA_TYPE", D.CLASS_NAME)
    val EFFECT = TextAttributesKey.createTextAttributesKey("GALA_EFFECT", D.KEYWORD)
    val QUANTUM = TextAttributesKey.createTextAttributesKey("GALA_QUANTUM", D.STATIC_METHOD)
    val GATE = TextAttributesKey.createTextAttributesKey("GALA_GATE", D.STATIC_METHOD)
    val BUILTIN = TextAttributesKey.createTextAttributesKey("GALA_BUILTIN", D.PREDEFINED_SYMBOL)
    val STRING = TextAttributesKey.createTextAttributesKey("GALA_STRING", D.STRING)
    val NUMBER = TextAttributesKey.createTextAttributesKey("GALA_NUMBER", D.NUMBER)
    val COMPLEX = TextAttributesKey.createTextAttributesKey("GALA_COMPLEX", D.NUMBER)
    val LINE_COMMENT = TextAttributesKey.createTextAttributesKey("GALA_LINE_COMMENT", D.LINE_COMMENT)
    val BLOCK_COMMENT = TextAttributesKey.createTextAttributesKey("GALA_BLOCK_COMMENT", D.BLOCK_COMMENT)
    val DOC_COMMENT = TextAttributesKey.createTextAttributesKey("GALA_DOC_COMMENT", D.DOC_COMMENT)
    val OPERATOR = TextAttributesKey.createTextAttributesKey("GALA_OPERATOR", D.OPERATION_SIGN)
    val PAREN = TextAttributesKey.createTextAttributesKey("GALA_PAREN", D.PARENTHESES)
    val BRACE = TextAttributesKey.createTextAttributesKey("GALA_BRACE", D.BRACES)
    val BRACKET = TextAttributesKey.createTextAttributesKey("GALA_BRACKET", D.BRACKETS)
    val COMMA = TextAttributesKey.createTextAttributesKey("GALA_COMMA", D.COMMA)
    val SEMICOLON = TextAttributesKey.createTextAttributesKey("GALA_SEMICOLON", D.SEMICOLON)
    val DOT = TextAttributesKey.createTextAttributesKey("GALA_DOT", D.DOT)
    val BAD_CHAR = TextAttributesKey.createTextAttributesKey("GALA_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER)
}

class GalaSyntaxHighlighter : SyntaxHighlighter {
    override fun getHighlightingLexer(): Lexer = GalaLexer()

    override fun getTokenHighlights(tokenType: IElementType?): Array<TextAttributesKey> {
        if (tokenType == null) return emptyArray()
        return when (tokenType) {
            GalaTokenTypes.KW_FN, GalaTokenTypes.KW_LET, GalaTokenTypes.KW_MUT,
            GalaTokenTypes.KW_IF, GalaTokenTypes.KW_ELSE, GalaTokenTypes.KW_MATCH,
            GalaTokenTypes.KW_FOR, GalaTokenTypes.KW_IN, GalaTokenTypes.KW_WHILE,
            GalaTokenTypes.KW_RETURN, GalaTokenTypes.KW_IMPORT, GalaTokenTypes.KW_FROM,
            GalaTokenTypes.KW_AS, GalaTokenTypes.KW_TYPE, GalaTokenTypes.KW_STRUCT,
            GalaTokenTypes.KW_ENUM, GalaTokenTypes.KW_TRAIT, GalaTokenTypes.KW_IMPL,
            GalaTokenTypes.KW_CONST, GalaTokenTypes.KW_WHERE -> arrayOf(GalaHighlightTokens.KEYWORD)

            GalaTokenTypes.KW_TRUE, GalaTokenTypes.KW_FALSE -> arrayOf(D.KEYWORD) // constant.language

            GalaTokenTypes.KW_MUT -> arrayOf(GalaHighlightTokens.DECLARATION)

            GalaTokenTypes.TYPE_QUBIT, GalaTokenTypes.TYPE_QUBITS,
            GalaTokenTypes.TYPE_MEASURED, GalaTokenTypes.TYPE_BOOL,
            GalaTokenTypes.TYPE_INT, GalaTokenTypes.TYPE_FLOAT,
            GalaTokenTypes.TYPE_COMPLEX, GalaTokenTypes.TYPE_VEC,
            GalaTokenTypes.TYPE_PARAMS, GalaTokenTypes.TYPE_STRING,
            GalaTokenTypes.TYPE_UNIT -> arrayOf(GalaHighlightTokens.TYPE)

            GalaTokenTypes.EFFECT_PURE, GalaTokenTypes.EFFECT_QUANTUM,
            GalaTokenTypes.EFFECT_PROB -> arrayOf(GalaHighlightTokens.EFFECT)

            GalaTokenTypes.KW_QUBIT, GalaTokenTypes.KW_QUBITS,
            GalaTokenTypes.KW_MEASURE, GalaTokenTypes.KW_REVERSE,
            GalaTokenTypes.KW_ADJOINT, GalaTokenTypes.KW_CONTROL,
            GalaTokenTypes.KW_GRAD, GalaTokenTypes.KW_DROP -> arrayOf(GalaHighlightTokens.QUANTUM)

            GalaTokenTypes.GATE_H, GalaTokenTypes.GATE_X, GalaTokenTypes.GATE_Y,
            GalaTokenTypes.GATE_Z, GalaTokenTypes.GATE_S, GalaTokenTypes.GATE_T,
            GalaTokenTypes.GATE_RX, GalaTokenTypes.GATE_RY, GalaTokenTypes.GATE_RZ,
            GalaTokenTypes.GATE_CX, GalaTokenTypes.GATE_CZ, GalaTokenTypes.GATE_SWAP -> arrayOf(GalaHighlightTokens.GATE)

            GalaTokenTypes.BUILTIN_PRINT, GalaTokenTypes.BUILTIN_ASSERT,
            GalaTokenTypes.BUILTIN_LEN, GalaTokenTypes.BUILTIN_SAMPLE -> arrayOf(GalaHighlightTokens.BUILTIN)

            GalaTokenTypes.STRING -> arrayOf(GalaHighlightTokens.STRING)
            GalaTokenTypes.INT, GalaTokenTypes.FLOAT,
            GalaTokenTypes.HEX, GalaTokenTypes.BIN -> arrayOf(GalaHighlightTokens.NUMBER)
            GalaTokenTypes.COMPLEX -> arrayOf(GalaHighlightTokens.COMPLEX)

            GalaTokenTypes.LINE_COMMENT -> arrayOf(GalaHighlightTokens.COMMENT)
            GalaTokenTypes.BLOCK_COMMENT -> arrayOf(GalaHighlightTokens.BLOCK_COMMENT)
            GalaTokenTypes.DOC_COMMENT -> arrayOf(GalaHighlightTokens.DOC_COMMENT)

            GalaTokenTypes.OP_PLUS, GalaTokenTypes.OP_MINUS,
            GalaTokenTypes.OP_STAR, GalaTokenTypes.OP_SLASH,
            GalaTokenTypes.OP_PERCENT, GalaTokenTypes.OP_EQEQ,
            GalaTokenTypes.OP_BANGEQ, GalaTokenTypes.OP_LT,
            GalaTokenTypes.OP_LE, GalaTokenTypes.OP_GT,
            GalaTokenTypes.OP_GE, GalaTokenTypes.OP_ANDAND,
            GalaTokenTypes.OP_OROR, GalaTokenTypes.OP_BANG,
            GalaTokenTypes.OP_EQ, GalaTokenTypes.OP_ARROW,
            GalaTokenTypes.OP_RANGE, GalaTokenTypes.OP_PIPE,
            GalaTokenTypes.OP_AND, GalaTokenTypes.OP_OR -> arrayOf(GalaHighlightTokens.OPERATOR)

            GalaTokenTypes.LPAREN, GalaTokenTypes.RPAREN -> arrayOf(GalaHighlightTokens.PAREN)
            GalaTokenTypes.LBRACE, GalaTokenTypes.RBRACE -> arrayOf(GalaHighlightTokens.BRACE)
            GalaTokenTypes.LBRACKET, GalaTokenTypes.RBRACKET -> arrayOf(GalaHighlightTokens.BRACKET)
            GalaTokenTypes.COMMA -> arrayOf(GalaHighlightTokens.COMMA)
            GalaTokenTypes.SEMICOLON -> arrayOf(GalaHighlightTokens.SEMICOLON)
            GalaTokenTypes.DOT -> arrayOf(GalaHighlightTokens.DOT)
            GalaTokenTypes.COLON -> arrayOf(GalaHighlightTokens.OPERATOR)
            GalaTokenTypes.ANGLE_LT, GalaTokenTypes.ANGLE_GT -> arrayOf(GalaHighlightTokens.OPERATOR)
            GalaTokenTypes.BAD_CHARACTER -> arrayOf(GalaHighlightTokens.BAD_CHAR)
            else -> emptyArray()
        }
    }

    class GalaLexer : Lexer {
        private val tokenTypes = mapOf(
            // keywords
            "fn" to GalaTokenTypes.KW_FN, "let" to GalaTokenTypes.KW_LET,
            "mut" to GalaTokenTypes.KW_MUT, "if" to GalaTokenTypes.KW_IF,
            "else" to GalaTokenTypes.KW_ELSE, "match" to GalaTokenTypes.KW_MATCH,
            "for" to GalaTokenTypes.KW_FOR, "in" to GalaTokenTypes.KW_IN,
            "while" to GalaTokenTypes.KW_WHILE, "return" to GalaTokenTypes.KW_RETURN,
            "import" to GalaTokenTypes.KW_IMPORT, "as" to GalaTokenTypes.KW_AS,
            "type" to GalaTokenTypes.KW_TYPE, "struct" to GalaTokenTypes.KW_STRUCT,
            "enum" to GalaTokenTypes.KW_ENUM, "trait" to GalaTokenTypes.KW_TRAIT,
            "impl" to GalaTokenTypes.KW_IMPL, "const" to GalaTokenTypes.KW_CONST,
            "where" to GalaTokenTypes.KW_WHERE,
            "true" to GalaTokenTypes.KW_TRUE, "false" to GalaTokenTypes.KW_FALSE,
            // effects
            "pure" to GalaTokenTypes.EFFECT_PURE,
            "quantum" to GalaTokenTypes.EFFECT_QUANTUM,
            "prob" to GalaTokenTypes.EFFECT_PROB,
            // types
            "Qubit" to GalaTokenTypes.TYPE_QUBIT, "Qubits" to GalaTokenTypes.TYPE_QUBITS,
            "Measured" to GalaTokenTypes.TYPE_MEASURED, "Bool" to GalaTokenTypes.TYPE_BOOL,
            "Int" to GalaTokenTypes.TYPE_INT, "Float" to GalaTokenTypes.TYPE_FLOAT,
            "Complex" to GalaTokenTypes.TYPE_COMPLEX, "Vec" to GalaTokenTypes.TYPE_VEC,
            "Params" to GalaTokenTypes.TYPE_PARAMS, "String" to GalaTokenTypes.TYPE_STRING,
            "Unit" to GalaTokenTypes.TYPE_UNIT, "Self" to GalaTokenTypes.TYPE_UNIT,
            // quantum keywords
            "qubit" to GalaTokenTypes.KW_QUBIT, "qubits" to GalaTokenTypes.KW_QUBITS,
            "measure" to GalaTokenTypes.KW_MEASURE, "reverse" to GalaTokenTypes.KW_REVERSE,
            "adjoint" to GalaTokenTypes.KW_ADJOINT, "control" to GalaTokenTypes.KW_CONTROL,
            "grad" to GalaTokenTypes.KW_GRAD, "drop" to GalaTokenTypes.KW_DROP,
            // gates
            "h" to GalaTokenTypes.GATE_H, "x" to GalaTokenTypes.GATE_X,
            "y" to GalaTokenTypes.GATE_Y, "z" to GalaTokenTypes.GATE_Z,
            "s" to GalaTokenTypes.GATE_S, "t" to GalaTokenTypes.GATE_T,
            "rx" to GalaTokenTypes.GATE_RX, "ry" to GalaTokenTypes.GATE_RY,
            "rz" to GalaTokenTypes.GATE_RZ, "cx" to GalaTokenTypes.GATE_CX,
            "cz" to GalaTokenTypes.GATE_CZ, "swap" to GalaTokenTypes.GATE_SWAP,
        )

        private var text: CharSequence = ""
        private var start = 0
        private var end = 0
        private var state = 0
        private var tokenStart = 0
        private var tokenEnd = 0
        private var tokenTypeValue: IElementType? = null

        override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
            text = buffer
            start = startOffset
            end = endOffset
            state = initialState
            advance()
        }

        override fun advance() {
            tokenStart = start
            if (start >= end) {
                tokenTypeValue = null
                return
            }
            val ch = text[start]
            when {
                ch.isWhitespace() -> {
                    while (start < end && text[start].isWhitespace()) start++
                    advance()
                }
                ch == '/' && start + 1 < end -> {
                    when (text[start + 1]) {
                        '/' -> {
                            start += 2
                            if (start < end && text[start - 1] == '/' && start > tokenStart + 2) {
                                // doc comment check handled by inspection
                            }
                            while (start < end && text[start] != '\n') start++
                            tokenEnd = start
                            tokenTypeValue = GalaTokenTypes.LINE_COMMENT
                        }
                        '*' -> {
                            start += 2
                            var depth = 1
                            while (start < end && depth > 0) {
                                if (text[start] == '/' && start + 1 < end && text[start + 1] == '*') {
                                    depth++
                                    start += 2
                                } else if (text[start] == '*' && start + 1 < end && text[start + 1] == '/') {
                                    depth--
                                    start += 2
                                } else {
                                    start++
                                }
                            }
                            tokenEnd = start
                            tokenTypeValue = GalaTokenTypes.BLOCK_COMMENT
                        }
                        else -> {
                            start++
                            tokenEnd = start
                            tokenTypeValue = GalaTokenTypes.OP_SLASH
                        }
                    }
                }
                ch == '"' -> {
                    start++
                    while (start < end) {
                        if (text[start] == '\\') start += 2
                        else if (text[start] == '"') break
                        else start++
                    }
                    if (start < end) start++
                    tokenEnd = start
                    tokenTypeValue = GalaTokenTypes.STRING
                }
                ch.isDigit() -> {
                    if (ch == '0' && start + 1 < end) {
                        when (text[start + 1]) {
                            'x', 'X' -> {
                                start += 2
                                while (start < end && text[start].isHexDigit()) start++
                                tokenEnd = start
                                tokenTypeValue = GalaTokenTypes.HEX
                                return
                            }
                            'b', 'B' -> {
                                start += 2
                                while (start < end && (text[start] == '0' || text[start] == '1')) start++
                                tokenEnd = start
                                tokenTypeValue = GalaTokenTypes.BIN
                                return
                            }
                        }
                    }
                    var isFloat = false
                    while (start < end && text[start].isDigit()) start++
                    if (start < end && text[start] == '.') {
                        isFloat = true
                        start++
                        while (start < end && text[start].isDigit()) start++
                    }
                    if (start < end && (text[start] == 'i' || text[start] == 'j')) {
                        start++
                        tokenEnd = start
                        tokenTypeValue = GalaTokenTypes.COMPLEX
                        return
                    }
                    tokenEnd = start
                    tokenTypeValue = if (isFloat) GalaTokenTypes.FLOAT else GalaTokenTypes.INT
                }
                ch == '_' || ch.isLetter() -> {
                    val wordStart = start
                    while (start < end && (text[start].isLetterOrDigit() || text[start] == '_')) start++
                    val word = text.substring(wordStart, start)
                    tokenEnd = start
                    tokenTypeValue = if (wordStart > 0 && text[wordStart - 1] == '/' && wordStart >= 3) {
                        val prefix = text.substring(wordStart - 3, wordStart)
                        if (prefix == "///") GalaTokenTypes.DOC_COMMENT else (tokenTypes[word] ?: GalaTokenTypes.IDENT)
                    } else {
                        tokenTypes[word] ?: GalaTokenTypes.IDENT
                    }
                }
                ch == '+' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_PLUS }
                ch == '-' -> {
                    if (start + 1 < end && text[start + 1] == '>') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_ARROW }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_MINUS }
                }
                ch == '*' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_STAR }
                ch == '%' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_PERCENT }
                ch == '=' -> {
                    if (start + 1 < end && text[start + 1] == '=') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_EQEQ }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_EQ }
                }
                ch == '!' -> {
                    if (start + 1 < end && text[start + 1] == '=') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_BANGEQ }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_BANG }
                }
                ch == '<' -> {
                    if (start + 1 < end && text[start + 1] == '=') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_LE }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_LT }
                }
                ch == '>' -> {
                    if (start + 1 < end && text[start + 1] == '=') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_GE }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_GT }
                }
                ch == '&' -> {
                    if (start + 1 < end && text[start + 1] == '&') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_ANDAND }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_AND }
                }
                ch == '|' -> {
                    if (start + 1 < end && text[start + 1] == '|') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_OROR }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_OR }
                }
                ch == '.' -> {
                    if (start + 1 < end && text[start + 1] == '.') { start += 2; tokenEnd = start; tokenTypeValue = GalaTokenTypes.OP_RANGE }
                    else { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.DOT }
                }
                ch == '(' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.LPAREN }
                ch == ')' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.RPAREN }
                ch == '{' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.LBRACE }
                ch == '}' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.RBRACE }
                ch == '[' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.LBRACKET }
                ch == ']' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.RBRACKET }
                ch == ',' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.COMMA }
                ch == ';' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.SEMICOLON }
                ch == ':' -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.COLON }
                else -> { start++; tokenEnd = start; tokenTypeValue = GalaTokenTypes.BAD_CHARACTER }
            }
        }

        override fun getTokenType(): IElementType? = tokenTypeValue
        override fun getTokenStart() = tokenStart
        override fun getTokenEnd() = tokenEnd
        override fun getBufferEnd() = end
        override fun getState() = state
    }
}

object GalaTokenTypes {
    val KW_FN = IElementType("KW_FN", GalaLanguage.INSTANCE)
    val KW_LET = IElementType("KW_LET", GalaLanguage.INSTANCE)
    val KW_MUT = IElementType("KW_MUT", GalaLanguage.INSTANCE)
    val KW_IF = IElementType("KW_IF", GalaLanguage.INSTANCE)
    val KW_ELSE = IElementType("KW_ELSE", GalaLanguage.INSTANCE)
    val KW_MATCH = IElementType("KW_MATCH", GalaLanguage.INSTANCE)
    val KW_FOR = IElementType("KW_FOR", GalaLanguage.INSTANCE)
    val KW_IN = IElementType("KW_IN", GalaLanguage.INSTANCE)
    val KW_WHILE = IElementType("KW_WHILE", GalaLanguage.INSTANCE)
    val KW_RETURN = IElementType("KW_RETURN", GalaLanguage.INSTANCE)
    val KW_IMPORT = IElementType("KW_IMPORT", GalaLanguage.INSTANCE)
    val KW_FROM = IElementType("KW_FROM", GalaLanguage.INSTANCE)
    val KW_AS = IElementType("KW_AS", GalaLanguage.INSTANCE)
    val KW_TYPE = IElementType("KW_TYPE", GalaLanguage.INSTANCE)
    val KW_STRUCT = IElementType("KW_STRUCT", GalaLanguage.INSTANCE)
    val KW_ENUM = IElementType("KW_ENUM", GalaLanguage.INSTANCE)
    val KW_TRAIT = IElementType("KW_TRAIT", GalaLanguage.INSTANCE)
    val KW_IMPL = IElementType("KW_IMPL", GalaLanguage.INSTANCE)
    val KW_CONST = IElementType("KW_CONST", GalaLanguage.INSTANCE)
    val KW_WHERE = IElementType("KW_WHERE", GalaLanguage.INSTANCE)
    val KW_TRUE = IElementType("KW_TRUE", GalaLanguage.INSTANCE)
    val KW_FALSE = IElementType("KW_FALSE", GalaLanguage.INSTANCE)
    val KW_QUBIT = IElementType("KW_QUBIT", GalaLanguage.INSTANCE)
    val KW_QUBITS = IElementType("KW_QUBITS", GalaLanguage.INSTANCE)
    val KW_MEASURE = IElementType("KW_MEASURE", GalaLanguage.INSTANCE)
    val KW_REVERSE = IElementType("KW_REVERSE", GalaLanguage.INSTANCE)
    val KW_ADJOINT = IElementType("KW_ADJOINT", GalaLanguage.INSTANCE)
    val KW_CONTROL = IElementType("KW_CONTROL", GalaLanguage.INSTANCE)
    val KW_GRAD = IElementType("KW_GRAD", GalaLanguage.INSTANCE)
    val KW_DROP = IElementType("KW_DROP", GalaLanguage.INSTANCE)

    val TYPE_QUBIT = IElementType("TYPE_QUBIT", GalaLanguage.INSTANCE)
    val TYPE_QUBITS = IElementType("TYPE_QUBITS", GalaLanguage.INSTANCE)
    val TYPE_MEASURED = IElementType("TYPE_MEASURED", GalaLanguage.INSTANCE)
    val TYPE_BOOL = IElementType("TYPE_BOOL", GalaLanguage.INSTANCE)
    val TYPE_INT = IElementType("TYPE_INT", GalaLanguage.INSTANCE)
    val TYPE_FLOAT = IElementType("TYPE_FLOAT", GalaLanguage.INSTANCE)
    val TYPE_COMPLEX = IElementType("TYPE_COMPLEX", GalaLanguage.INSTANCE)
    val TYPE_VEC = IElementType("TYPE_VEC", GalaLanguage.INSTANCE)
    val TYPE_PARAMS = IElementType("TYPE_PARAMS", GalaLanguage.INSTANCE)
    val TYPE_STRING = IElementType("TYPE_STRING", GalaLanguage.INSTANCE)
    val TYPE_UNIT = IElementType("TYPE_UNIT", GalaLanguage.INSTANCE)

    val EFFECT_PURE = IElementType("EFFECT_PURE", GalaLanguage.INSTANCE)
    val EFFECT_QUANTUM = IElementType("EFFECT_QUANTUM", GalaLanguage.INSTANCE)
    val EFFECT_PROB = IElementType("EFFECT_PROB", GalaLanguage.INSTANCE)

    val GATE_H = IElementType("GATE_H", GalaLanguage.INSTANCE)
    val GATE_X = IElementType("GATE_X", GalaLanguage.INSTANCE)
    val GATE_Y = IElementType("GATE_Y", GalaLanguage.INSTANCE)
    val GATE_Z = IElementType("GATE_Z", GalaLanguage.INSTANCE)
    val GATE_S = IElementType("GATE_S", GalaLanguage.INSTANCE)
    val GATE_T = IElementType("GATE_T", GalaLanguage.INSTANCE)
    val GATE_RX = IElementType("GATE_RX", GalaLanguage.INSTANCE)
    val GATE_RY = IElementType("GATE_RY", GalaLanguage.INSTANCE)
    val GATE_RZ = IElementType("GATE_RZ", GalaLanguage.INSTANCE)
    val GATE_CX = IElementType("GATE_CX", GalaLanguage.INSTANCE)
    val GATE_CZ = IElementType("GATE_CZ", GalaLanguage.INSTANCE)
    val GATE_SWAP = IElementType("GATE_SWAP", GalaLanguage.INSTANCE)

    val BUILTIN_PRINT = IElementType("BUILTIN_PRINT", GalaLanguage.INSTANCE)
    val BUILTIN_ASSERT = IElementType("BUILTIN_ASSERT", GalaLanguage.INSTANCE)
    val BUILTIN_LEN = IElementType("BUILTIN_LEN", GalaLanguage.INSTANCE)
    val BUILTIN_SAMPLE = IElementType("BUILTIN_SAMPLE", GalaLanguage.INSTANCE)

    val IDENT = IElementType("IDENT", GalaLanguage.INSTANCE)
    val STRING = IElementType("STRING", GalaLanguage.INSTANCE)
    val INT = IElementType("INT", GalaLanguage.INSTANCE)
    val FLOAT = IElementType("FLOAT", GalaLanguage.INSTANCE)
    val HEX = IElementType("HEX", GalaLanguage.INSTANCE)
    val BIN = IElementType("BIN", GalaLanguage.INSTANCE)
    val COMPLEX = IElementType("COMPLEX", GalaLanguage.INSTANCE)

    val LINE_COMMENT = IElementType("LINE_COMMENT", GalaLanguage.INSTANCE)
    val BLOCK_COMMENT = IElementType("BLOCK_COMMENT", GalaLanguage.INSTANCE)
    val DOC_COMMENT = IElementType("DOC_COMMENT", GalaLanguage.INSTANCE)

    val OP_PLUS = IElementType("OP_PLUS", GalaLanguage.INSTANCE)
    val OP_MINUS = IElementType("OP_MINUS", GalaLanguage.INSTANCE)
    val OP_STAR = IElementType("OP_STAR", GalaLanguage.INSTANCE)
    val OP_SLASH = IElementType("OP_SLASH", GalaLanguage.INSTANCE)
    val OP_PERCENT = IElementType("OP_PERCENT", GalaLanguage.INSTANCE)
    val OP_EQEQ = IElementType("OP_EQEQ", GalaLanguage.INSTANCE)
    val OP_BANGEQ = IElementType("OP_BANGEQ", GalaLanguage.INSTANCE)
    val OP_LT = IElementType("OP_LT", GalaLanguage.INSTANCE)
    val OP_LE = IElementType("OP_LE", GalaLanguage.INSTANCE)
    val OP_GT = IElementType("OP_GT", GalaLanguage.INSTANCE)
    val OP_GE = IElementType("OP_GE", GalaLanguage.INSTANCE)
    val OP_ANDAND = IElementType("OP_ANDAND", GalaLanguage.INSTANCE)
    val OP_OROR = IElementType("OP_OROR", GalaLanguage.INSTANCE)
    val OP_BANG = IElementType("OP_BANG", GalaLanguage.INSTANCE)
    val OP_AND = IElementType("OP_AND", GalaLanguage.INSTANCE)
    val OP_OR = IElementType("OP_OR", GalaLanguage.INSTANCE)
    val OP_EQ = IElementType("OP_EQ", GalaLanguage.INSTANCE)
    val OP_ARROW = IElementType("OP_ARROW", GalaLanguage.INSTANCE)
    val OP_RANGE = IElementType("OP_RANGE", GalaLanguage.INSTANCE)
    val OP_PIPE = IElementType("OP_PIPE", GalaLanguage.INSTANCE)

    val LPAREN = IElementType("LPAREN", GalaLanguage.INSTANCE)
    val RPAREN = IElementType("RPAREN", GalaLanguage.INSTANCE)
    val LBRACE = IElementType("LBRACE", GalaLanguage.INSTANCE)
    val RBRACE = IElementType("RBRACE", GalaLanguage.INSTANCE)
    val LBRACKET = IElementType("LBRACKET", GalaLanguage.INSTANCE)
    val RBRACKET = IElementType("RBRACKET", GalaLanguage.INSTANCE)
    val COMMA = IElementType("COMMA", GalaLanguage.INSTANCE)
    val SEMICOLON = IElementType("SEMICOLON", GalaLanguage.INSTANCE)
    val DOT = IElementType("DOT", GalaLanguage.INSTANCE)
    val COLON = IElementType("COLON", GalaLanguage.INSTANCE)
    val ANGLE_LT = IElementType("ANGLE_LT", GalaLanguage.INSTANCE)
    val ANGLE_GT = IElementType("ANGLE_GT", GalaLanguage.INSTANCE)
    val BAD_CHARACTER = IElementType("BAD_CHARACTER", GalaLanguage.INSTANCE)
}

object GalaKeywords {
    val KEYWORDS = setOf(
        "fn", "let", "mut", "if", "else", "match", "for",
        "in", "while", "return", "import", "as", "type",
        "struct", "enum", "trait", "impl", "const", "where",
        "true", "false"
    )
    val TYPES = setOf(
        "Qubit", "Qubits", "Measured", "Bool",
        "Int", "Float", "Complex", "Vec", "Params",
        "String", "Unit", "Self"
    )
    val EFFECTS = setOf("pure", "quantum", "prob")
    val QUANTUM = setOf("qubit", "qubits", "measure", "reverse", "adjoint", "control", "grad", "drop")
    val GATES = setOf("h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz", "swap")
}