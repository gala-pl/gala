//! Incremental compilation with salsa.
//!
//! A `SourceModule` is an input holding a file's text. `check_module` is a tracked query: salsa
//! caches its result and re-runs it only when the text changes. Editors and the language server
//! use this to recheck a file without redoing unchanged work.

use crate::{compile_source, Sources};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The text of one source file.
#[salsa::input]
pub struct SourceModule {
    #[returns(deref)]
    pub text: String,
}

/// The query database interface.
#[salsa::db]
pub trait Db: salsa::Database {}

/// The concrete database.
#[salsa::db]
#[derive(Default, Clone)]
pub struct GalaDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for GalaDatabase {}

#[salsa::db]
impl Db for GalaDatabase {}

/// How many times `check_module` has actually run (cache misses). Used by tests.
pub static CHECK_RUNS: AtomicUsize = AtomicUsize::new(0);

/// The error codes of a module, in order. Cached until the module's text changes.
#[salsa::tracked]
pub fn check_module(db: &dyn Db, module: SourceModule) -> Vec<String> {
    CHECK_RUNS.fetch_add(1, Ordering::SeqCst);
    let mut sources = Sources::default();
    match compile_source(module.text(db), &mut sources) {
        Ok(_) => Vec::new(),
        Err(d) => d.errors().iter().map(|e| e.code.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    #[test]
    fn unchanged_text_is_served_from_the_cache_and_changes_rerun_the_query() {
        let mut db = GalaDatabase::default();
        let module = SourceModule::new(&db, "fn main() -> Int { 0 }".to_string());
        assert!(check_module(&db, module).is_empty());
        let after_first = CHECK_RUNS.load(Ordering::SeqCst);

        // Same revision, same input: the cached value is returned without running the query.
        assert!(check_module(&db, module).is_empty());
        assert_eq!(CHECK_RUNS.load(Ordering::SeqCst), after_first);

        // A new text is a new input revision: the query runs again and reports the new error.
        module.set_text(&mut db).to("fn main() -> Int { missing }".to_string());
        assert_eq!(check_module(&db, module), vec!["E0202".to_string()]);
        assert_eq!(CHECK_RUNS.load(Ordering::SeqCst), after_first + 1);
    }
}
