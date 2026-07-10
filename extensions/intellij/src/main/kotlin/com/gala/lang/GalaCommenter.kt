package com.gala.lang

import com.intellij.lang.Commenter

class GalaCommenter : Commenter {
    override fun getLineCommentPrefix() = "//"
    override fun getBlockCommentPrefix() = "/*"
    override fun getBlockCommentSuffix() = "*/"
    override fun getCommentedBlockCommentPrefix() = "/*"
    override fun getCommentedBlockCommentSuffix() = "*/"
}