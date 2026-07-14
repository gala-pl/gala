fn main() {
    let sources = vec![
        ("type_qubit", "fn f(q: Qubit) -> Qubit {\n    return q;\n}\n"),
        ("type_bool", "fn f() -> Bool { true }\n"),
        ("return_q", "fn f() { return q; }\n"),
        ("simple_q", "fn f(q: Qubit) -> Qubit { return q; }"),
    ];
    for (name, source) in &sources {
        let mut map = gala_span::SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        match gala_parser::parse_file(fid, source, &mut map) {
            Ok(items) => println!("  OK {name}: {} items", items.len()),
            Err(diags) => {
                let msgs: Vec<_> = diags.diagnostics.iter().map(|d| format!("{}: {}", d.code, d.message)).collect();
                println!("  FAIL {name}: {}", msgs.join("; "));
            }
        }
    }
}
