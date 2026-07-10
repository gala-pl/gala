//! Classical codegen (Cranelift/LLVM) for Gala.

use gala_gir::{Gir, GirFunc, BlockId};
use gala_diagnostics::{Diagnostic, Diagnostics, codes};

/// Target for classical codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalTarget {
    Cranelift,
    Llvm,
}

/// Emit classical machine code from GIR.
pub fn emit_classical(gir: &Gir, target: ClassicalTarget) -> Result<Vec<u8>, Diagnostics> {
    let _ = (gir, target);
    Err(Diagnostics::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classical_targets() {
        assert_eq!(ClassicalTarget::Cranelift as u8, 0);
        assert_eq!(ClassicalTarget::Llvm as u8, 1);
    }

    #[test]
    fn test_emit_classical_returns_error() {
        let gir = Gir::default();
        let result = emit_classical(&gir, ClassicalTarget::Cranelift);
        // Currently returns error since not implemented
        assert!(result.is_err());
    }

    #[test]
    fn test_classical_target_debug() {
        let cranelift = ClassicalTarget::Cranelift;
        let llvm = ClassicalTarget::Llvm;
        assert!(format!("{:?}", cranelift).contains("Cranelift"));
        assert!(format!("{:?}", llvm).contains("Llvm"));
    }
}