#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let code = gala_diagnostics::DiagnosticCode(source.len() as u16 % 999);
        let _diag = gala_diagnostics::Diagnostic::error(code, source)
            .with_note("fuzz-generated diagnostic");
    }
});
