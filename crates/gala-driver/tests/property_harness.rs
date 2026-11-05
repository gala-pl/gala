//! Property harness conformance: run `gala test --property` on files with
//! `// property-expect: ...` header lines.

use std::path::PathBuf;

fn find_gala_binary() -> PathBuf {
    // Try to find the gala binary in the target directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_dir = manifest_dir.join("../../target/debug");
    let binary = target_dir.join("gala");
    if binary.exists() {
        return binary;
    }
    // Fallback: try current directory
    let binary = PathBuf::from("./target/debug/gala");
    if binary.exists() {
        return binary;
    }
    panic!("gala binary not found in target/debug or ./target/debug");
}

fn property_expect(text: &str) -> Option<Vec<String>> {
    let mut expect = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("//") else { continue };
        let rest = rest.trim();
        if let Some(v) = rest.strip_prefix("property-expect:") {
            expect.push(v.trim().to_string());
        }
    }
    if expect.is_empty() {
        None
    } else {
        Some(expect)
    }
}

#[test]
fn property_harness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/valid/property_tests.gala");
    let text = fs::read_to_string(&root).expect("read property_tests.gala");
    let Some(expected) = property_expect(&text) else {
        return; // no expectation, skip
    };

    let gala = find_gala_binary();
    let output = Command::new(gala)
        .args(["test", "--property", "--json"])
        .arg(&root)
        .output()
        .expect("run gala test --property");

    assert!(
        output.status.success(),
        "gala test --property failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    let failed = json["failed"].as_u64().unwrap_or(0);
    let json_str = json.to_string();
    assert_eq!(failed, 0, "some property tests failed: {json_str}");

    // Check that all expected properties passed
    let results = json["results"].as_array().expect("results array");
    for exp in &expected {
        let found = results.iter().any(|r| {
            r["property"].as_str() == Some(exp.as_str()) && r["passed"].as_bool() == Some(true)
        });
        assert!(found, "expected property {} to pass", exp);
    }
}

use std::fs;
use std::path::Path;
use std::process::Command;
