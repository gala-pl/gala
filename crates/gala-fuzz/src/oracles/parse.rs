use gala_driver::GalaDatabase;

/// Assert that parsing `source` does not panic.
/// Returns the parse result (may contain errors — the point is no ICE).
pub fn must_not_panic(db: &mut GalaDatabase, source: &str) {
    let fid = db.add_file("fuzz.gala".into(), source.to_string());
    let _ = db.parse(fid);
}
