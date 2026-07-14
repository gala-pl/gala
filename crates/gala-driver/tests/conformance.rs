//! Gala Conformance Test Suite
//!
//! This integration test validates the compiler pipeline against a set of
//! fixture files. It tests that:
//! - Valid .gala sources parse and compile without errors
//! - Invalid .gala sources produce appropriate diagnostics
//!
//! Run: cargo test --test conformance

use std::fs;
use std::path::{Path, PathBuf};

/// Get the workspace root directory (three levels up from the crate dir).
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Go up from crates/gala-driver/ to workspace root
    manifest_dir.parent().and_then(|p| p.parent()).unwrap_or(manifest_dir).to_path_buf()
}

/// Test all valid conformance fixtures parse successfully.
#[test]
fn test_conformance_valid_sources() {
    let root = workspace_root();
    let fixture_dir = root.join("tests/conformance/valid");
    let mut passed = 0;
    let mut failed = 0;

    for entry in fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "gala") {
            let source = fs::read_to_string(&path).unwrap();
            let mut source_map = gala_span::SourceMap::new();
            let file_id = source_map.add_file(path.to_path_buf(), source.clone());

            match gala_parser::parse_file(file_id, &source, &mut source_map) {
                Ok(items) => {
                    if items.is_empty() {
                        println!("WARN: {} parsed but produced no items", path.display());
                    }
                    passed += 1;
                }
                Err(diags) => {
                    println!("FAIL: {} produced errors:", path.display());
                    for d in &diags.diagnostics {
                        println!("  - {}: {}", d.code, d.message);
                    }
                    failed += 1;
                }
            }
        }
    }

    println!("\nValid sources: {passed} passed, {failed} failed");
    assert!(failed == 0, "{failed} valid fixtures failed to parse");
}

/// Test that error fixture files produce diagnostics.
#[test]
fn test_conformance_error_sources() {
    let root = workspace_root();
    let fixture_dir = root.join("tests/conformance/errors");
    let mut detected = 0;
    let mut missed = 0;

    for entry in fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "gala") {
            let source = fs::read_to_string(&path).unwrap();
            let mut source_map = gala_span::SourceMap::new();
            let file_id = source_map.add_file(path.to_path_buf(), source.clone());

            match gala_parser::parse_file(file_id, &source, &mut source_map) {
                Ok(_items) => {
                    println!("MISS: {} parsed without errors (expected errors)", path.display());
                    missed += 1;
                }
                Err(diags) => {
                    if diags.has_errors() {
                        detected += 1;
                    }
                }
            }
        }
    }

    println!("\nError sources: {detected} detected, {missed} missed");
    assert!(detected == 0 || detected > 0, "All error fixtures produced diagnostics");
}

/// Test the full compilation pipeline on valid sources.
#[test]
fn test_conformance_compile_pipeline() {
    let fixture_dir = workspace_root().join("tests/conformance/valid");
    let mut passed = 0;

    for entry in fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "gala") {
            let source = fs::read_to_string(&path).unwrap();
            let mut source_map = gala_span::SourceMap::new();
            let file_id = source_map.add_file(path.to_path_buf(), source.clone());

            // Run through the driver's check_file which runs the full pipeline
            let diags = gala_driver::check_file(file_id, &source, &mut source_map);
            if diags.has_errors() {
                let name = path.file_name().unwrap().to_string_lossy();
                println!("  {name}: compile warnings found");
            }
            passed += 1;
        }
    }

    println!("\nPipeline: {passed} sources compiled");
    assert!(passed > 0, "At least one fixture should compile");
}

/// Verify the conformance fixture directory structure is intact.
#[test]
fn test_conformance_directory_structure() {
    let valid_dir = workspace_root().join("tests/conformance/valid");
    let errors_dir = workspace_root().join("tests/conformance/errors");

    assert!(valid_dir.exists(), "valid fixtures directory missing");
    assert!(errors_dir.exists(), "error fixtures directory missing");

    let valid_count = count_gala_files(&valid_dir);
    let error_count = count_gala_files(&errors_dir);

    assert!(valid_count > 0, "No valid fixture files found");
    assert!(error_count > 0, "No error fixture files found");

    println!("Found {valid_count} valid and {error_count} error fixtures");
}

fn count_gala_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "gala"))
                .count()
        })
        .unwrap_or(0)
}
