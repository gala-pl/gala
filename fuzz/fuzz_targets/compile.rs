#![no_main]

use gala_fuzz::oracles::compile::must_not_ice;
use gala_driver::GalaDatabase;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let mut db = GalaDatabase::new();
        must_not_ice(&mut db, source);
    }
});
