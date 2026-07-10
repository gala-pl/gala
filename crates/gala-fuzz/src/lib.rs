//! Fuzzing infrastructure for the Gala compiler.

/// Generate a random Gala source program for fuzzing.
pub fn generate_random_source(rng: &mut impl rand::Rng) -> String {
    let keywords = ["fn", "let", "if", "else", "return", "match", "for", "while"];
    let types = ["Int", "Float", "Bool", "String", "Unit", "Qubit"];
    let ops = ["+", "-", "*", "/", "==", "<", ">", "&&", "||"];

    let mut src = String::new();
    let num_fns = rng.gen_range(1..=3);
    for fi in 0..num_fns {
        src.push_str(&format!("fn func{}(", fi));
        let num_params = rng.gen_range(0..=3);
        for pi in 0..num_params {
            if pi > 0 { src.push_str(", "); }
            let ty = types[rng.gen_range(0..types.len())];
            src.push_str(&format!("p{pi}: {ty}"));
        }
        src.push_str(&format!(") -> Int {{\n"));
        src.push_str("    return 42;\n");
        src.push_str("}\n\n");
    }
    src
}

/// Check if a generated program compiles without panic.
pub fn fuzz_compile(source: &str, f: &mut dyn FnMut(&str)) {
    f(source);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_source_contains_fn() {
        let mut rng = rand::thread_rng();
        let src = generate_random_source(&mut rng);
        assert!(src.contains("fn func"));
        assert!(src.contains("return 42"));
    }

    #[test]
    fn test_generate_random_source_valid_syntax() {
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let src = generate_random_source(&mut rng);
            // Should have balanced braces
            let opens = src.matches('{').count();
            let closes = src.matches('}').count();
            assert_eq!(opens, closes, "unbalanced braces in: {src}");
        }
    }
}