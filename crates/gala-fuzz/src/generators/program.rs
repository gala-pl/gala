use proptest::strategy::{BoxedStrategy, Strategy};

/// Returns a strategy that generates random valid Gala source snippets.
pub fn any_valid_program() -> BoxedStrategy<String> {
    proptest::string::string_regex(r"(fn main() -> Int \{ (print\(\d+\))? return 0; \})|(fn bell\(\) -> Qubits<2> quantum \{ let a = qubit\(\); let b = qubit\(\); h\(a\); cx\(a, b\); \(a, b\) \})")
        .expect("valid regex")
        .boxed()
}

/// Returns a strategy that generates arbitrary byte sequences (may be invalid Gala).
pub fn any_bytes() -> BoxedStrategy<Vec<u8>> {
    proptest::collection::vec(proptest::arbitrary::any::<u8>(), 0..1024).boxed()
}
