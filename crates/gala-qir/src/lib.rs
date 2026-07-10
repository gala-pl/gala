//! QIR emission (LLVM IR) for Gala.

use gala_gir::{Gir, GirFunc};
use gala_span::Span;
use gala_diagnostics::{Diagnostic, Diagnostics, codes};

/// QIR profile to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QirProfile {
    Base,
    Adaptive,
}

/// Emit QIR from GIR.
pub fn emit_qir(gir: &Gir, profile: QirProfile) -> Result<String, Diagnostics> {
    let mut diags = Diagnostics::new();
    let mut output = String::new();

    output.push_str("; QIR Module\n");
    output.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

    for func in gir.funcs.values() {
        output.push_str(&format!("; Function: {:?}\n", func.name));
        output.push_str("define void @main() {\n");
        output.push_str("  ret void\n");
        output.push_str("}\n\n");
    }

    let _ = profile;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qir_emission_basic() {
        let gir = Gir::default();
        let result = emit_qir(&gir, QirProfile::Base);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("QIR Module"));
    }

    #[test]
    fn test_qir_profile_debug() {
        assert_eq!(format!("{:?}", QirProfile::Base), "Base");
        assert_eq!(format!("{:?}", QirProfile::Adaptive), "Adaptive");
    }

    #[test]
    fn test_qir_emission_both_profiles() {
        let gir = Gir::default();
        for profile in &[QirProfile::Base, QirProfile::Adaptive] {
            let result = emit_qir(&gir, *profile);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_qir_emission_with_func() {
        let mut gir = Gir::default();
        let func = GirFunc::new_test();
        gir.funcs.insert(func.id, func);
        let result = emit_qir(&gir, QirProfile::Base);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test"));
    }

    #[test]
    fn test_qir_target_triple() {
        let result = emit_qir(&Gir::default(), QirProfile::Base);
        let output = result.unwrap();
        assert!(output.contains("x86_64"));
    }
}