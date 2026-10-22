//! Multi-file packages: imports resolve to modules under the package's `src/` directory.

use gala_driver::{compile_file, Sources};
use std::fs;
use std::path::{Path, PathBuf};

/// Write a small package into a fresh directory under the system temp dir.
fn package(tag: &str, main: &str, module: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("gala-multifile-{tag}-{}", std::process::id()));
    fs::create_dir_all(root.join("src")).expect("create package");
    fs::write(root.join("src").join("main.gala"), main).expect("write main");
    fs::write(root.join("src").join("mylib.gala"), module).expect("write module");
    root
}

fn errors(path: &Path) -> Vec<String> {
    let mut sources = Sources::default();
    match compile_file(path, &mut sources) {
        Ok(_) => Vec::new(),
        Err(d) => d.errors().iter().map(|e| e.code.to_string()).collect(),
    }
}

#[test]
fn import_resolves_to_a_sibling_module() {
    let root = package(
        "ok",
        "import mylib.{ helper }\n\nfn main() -> Int {\n    helper()\n}\n",
        "fn helper() -> Int {\n    7\n}\n",
    );
    assert!(errors(&root.join("src").join("main.gala")).is_empty());
}

#[test]
fn missing_name_in_a_module_is_reported_at_the_import() {
    let root = package(
        "missing",
        "import mylib.{ absent }\n\nfn main() -> Int {\n    0\n}\n",
        "fn helper() -> Int {\n    7\n}\n",
    );
    assert!(errors(&root.join("src").join("main.gala")).contains(&"E0202".to_string()));
}
