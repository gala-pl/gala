use gala_driver::GalaDatabase;

/// Assert that the full compilation pipeline for `source` does not ICE.
/// Diagnostics may be produced — the point is no panic/unwinding.
pub fn must_not_ice(db: &mut GalaDatabase, source: &str) {
    let fid = db.add_file("fuzz.gala".into(), source.to_string());
    let _ = db.parse(fid);
}
