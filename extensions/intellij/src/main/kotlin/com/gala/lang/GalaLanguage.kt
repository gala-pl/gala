package com.gala.lang

import com.intellij.lang.Language
import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

class GalaLanguage private constructor() : Language("gala") {
    companion object {
        val INSTANCE = GalaLanguage()
    }

    override fun getDisplayName() = "Gala"
    override fun isCaseSensitive() = true
}

class GalaFileType : LanguageFileType(GalaLanguage.INSTANCE) {
    companion object {
        val INSTANCE = GalaFileType()
    }

    override fun getName() = "Gala"
    override fun getDescription() = "Gala source file (.gala)"
    override fun getDefaultExtension() = "gala"
    override fun getIcon(): Icon? = null
}