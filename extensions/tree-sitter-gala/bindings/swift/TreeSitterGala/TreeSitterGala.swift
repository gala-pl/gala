import TreeSitter

/// Tree-sitter language binding for Gala
public let tree_sitter_gala: ParserLanguage = {
    let language = unsafeUninitializedLanguage()
    // The parser C source is generated from grammar.js
    return ParserLanguage(language)
}()

public class GalaLanguage: Language {
    public override init() {
        super.init()
        self.language = tree_sitter_gala
    }
}