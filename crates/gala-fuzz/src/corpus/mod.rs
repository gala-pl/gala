use std::path::Path;

/// Names of seed programs bundled with the repo.
pub const SEED_PROGRAMS: &[&str] = &[
    "// Bell pair\nfn bell() -> Qubits<2> quantum {\n    let a = qubit();\n    let b = qubit();\n    h(a);\n    let (a, b) = cx(a, b);\n    (a, b)\n}\n",
    "// Hello world\nfn main() -> Int {\n    let msg = \"Hello, Gala!\";\n    print(msg);\n    return 0;\n}\n",
    "// Classical fib\nfn fib(n: Int) -> Int pure {\n    if n <= 1 { return n; }\n    return fib(n - 1) + fib(n - 2);\n}\nfn main() -> Int pure {\n    print(fib(10));\n    return 0;\n}\n",
];

/// Path to the corpus directory for a given fuzz target.
pub fn corpus_dir(target: &str) -> String {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("gala-fuzz is three levels deep from repo root");
    format!("{}/fuzz/corpus/{}", repo_root.display(), target)
}
