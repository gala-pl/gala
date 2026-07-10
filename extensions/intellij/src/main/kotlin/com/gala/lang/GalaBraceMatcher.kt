package com.gala.lang

import com.intellij.lang.BraceMatcher
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet

class GalaBraceMatcher : PairedBraceMatcher {
    override fun getPairs() = arrayOf(
        GalaBracePair(GalaTokenTypes.LBRACE, GalaTokenTypes.RBRACE, true),
        GalaBracePair(GalaTokenTypes.LPAREN, GalaTokenTypes.RPAREN, false),
        GalaBracePair(GalaTokenTypes.LBRACKET, GalaTokenTypes.RBRACKET, false)
    )

    override fun isStructuralBrace(tokenType: IElementType) =
        tokenType == GalaTokenTypes.LBRACE || tokenType == GalaTokenTypes.RBRACE
}

class GalaBracePair(
    private val left: IElementType,
    private val right: IElementType,
    private val structural: Boolean
) : BraceMatcher.Pair {
    override fun getLeftBraceType() = left
    override fun getRightBraceType() = right
    override fun isStructuralPair() = structural
}