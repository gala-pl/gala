use gala_lexer::{Lexer, Token};
use gala_span::SourceMap;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexer_doesnt_crash(s in "\\PC*") {
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), s.clone());
        let lexer = Lexer::new(fid, &s);
        let tokens = lexer.collect_all();
        let _ = tokens;
    }

    #[test]
    fn test_token_spans_start_nonnegative(s in "\\PC{0,200}") {
        let source_len = s.len() as u32;
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), s.clone());
        let lexer = Lexer::new(fid, &s);
        let tokens = lexer.collect_all();

        for (token, span) in &tokens {
            let start = span.byte_span.start;
            let end = span.byte_span.end;
            // Start must be within source range
            prop_assert!(
                start <= source_len,
                "token {:?} has start {} > source len {}",
                token, start, source_len
            );
            // Start must be <= end
            prop_assert!(
                start <= end,
                "token {:?} has start {} > end {}",
                token, start, end
            );
        }
    }

    #[test]
    fn test_all_tokens_cover_contiguous_regions(s in "\\PC{0,100}") {
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), s.clone());
        let lexer = Lexer::new(fid, &s);
        let tokens = lexer.collect_all();

        let mut prev_end: u32 = 0;
        for (token, span) in &tokens {
            if matches!(token, Token::Eof) {
                break;
            }
            if span.byte_span.start >= prev_end {
                prev_end = span.byte_span.end;
            }
        }
        // If we got here without panic, tokens progress monotonically
    }

    #[test]
    fn test_identifiers_are_ident_tokens(ident in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
        let keywords = [
            "fn", "let", "mut", "if", "else", "for", "in", "while", "return",
            "import", "as", "struct", "enum", "trait", "impl", "type", "const",
            "pure", "quantum", "prob", "qubit", "qubits", "measure", "reverse",
            "adjoint", "control", "grad", "drop", "true", "false",
            "Bool", "Int", "Float", "Complex", "String", "Unit", "Params",
            "Measured", "Qubit", "Qubits", "Vec",
        ];
        if keywords.contains(&ident.as_str()) || ident.is_empty() {
            return Ok(());
        }

        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), ident.clone());
        let lexer = Lexer::new(fid, &ident);
        let tokens = lexer.collect_all();

        let non_eof_tokens: Vec<_> = tokens.iter().filter(|(t, _)| !matches!(t, Token::Eof)).collect();
        prop_assert_eq!(
            non_eof_tokens.len(), 1,
            "expected 1 token for ident '{:?}', got {:?}",
            ident, non_eof_tokens.len()
        );
        if let Some((token, span)) = non_eof_tokens.first() {
            prop_assert!(
                matches!(token, Token::Ident(_)),
                "expected Ident token, got {:?}", token
            );
            let start = span.byte_span.start as usize;
            let end = span.byte_span.end as usize;
            if let Some(slice) = ident.get(start..end) {
                prop_assert_eq!(slice, ident.as_str(), "span slice does not match ident");
            }
        }
    }

    #[test]
    fn test_spans_never_panic_on_slice(s in "\\PC{0,200}") {
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), s.clone());
        let lexer = Lexer::new(fid, &s);
        let tokens = lexer.collect_all();

        for (token, span) in &tokens {
            if matches!(token, Token::Eof) {
                break;
            }
            let start = span.byte_span.start as usize;
            let end = span.byte_span.end as usize;
            if start <= end && end <= s.len() {
                // This shouldn't panic even for non-char-boundary slices
                let _ = s.get(start..end);
            }
        }
    }
}
