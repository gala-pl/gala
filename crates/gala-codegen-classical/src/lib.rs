//! Classical codegen (Cranelift/LLVM) for Gala.

use gala_diagnostics::Diagnostics;
use gala_gir::Gir;

mod cranelift_backend;
pub mod llvm_codegen;

pub use cranelift_backend::emit_cranelift;
pub use llvm_codegen::{
    emit_llvm, emit_native, emit_native_executable, emit_native_object, EmitTarget,
};

/// Target for classical codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalTarget {
    Cranelift,
    Llvm,
}

/// Emit classical machine code from GIR.
pub fn emit_classical(gir: &Gir, target: ClassicalTarget) -> Result<Vec<u8>, Diagnostics> {
    match target {
        ClassicalTarget::Llvm => emit_llvm(gir).map(String::into_bytes),
        ClassicalTarget::Cranelift => emit_cranelift(gir),
    }
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
    fn test_emit_classical_cranelift_returns_object() {
        let gir = Gir::default();
        let result = emit_classical(&gir, ClassicalTarget::Cranelift);
        assert!(result.is_ok());
        assert!(!result.expect("empty Cranelift module should emit").is_empty());
    }

    #[test]
    fn test_cranelift_lowers_classical_arithmetic() {
        use gala_gir::{
            BlockId, ClassicalOp, Constant, GirFunc, GirNode, NodeId, Terminator, ValueId,
        };
        use gala_span::Span;

        let mut gir = Gir::default();
        let mut func = GirFunc::new_test();
        let block = func.blocks.get_mut(&BlockId(0)).expect("test function has an entry block");
        block.nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        block.terminator = Some(Terminator::Return(Some(ValueId(2))));
        gir.nodes.insert(
            NodeId(0),
            GirNode {
                id: NodeId(0),
                kind: gala_gir::NodeKind::Constant(Constant::Int(7)),
                operands: Vec::new(),
                results: vec![ValueId(0)],
                span: Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(1),
            GirNode {
                id: NodeId(1),
                kind: gala_gir::NodeKind::Constant(Constant::Int(5)),
                operands: Vec::new(),
                results: vec![ValueId(1)],
                span: Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(2),
            GirNode {
                id: NodeId(2),
                kind: gala_gir::NodeKind::Classical(ClassicalOp::Add),
                operands: vec![ValueId(0), ValueId(1)],
                results: vec![ValueId(2)],
                span: Span::dummy(),
            },
        );
        gir.funcs.insert(func.id, func);
        let object = emit_cranelift(&gir).expect("classical GIR should lower through Cranelift");
        assert!(!object.is_empty());
    }

    #[test]
    fn test_emit_classical_llvm_returns_ir() {
        let result = emit_classical(&Gir::default(), ClassicalTarget::Llvm);
        assert!(result.is_ok());
        assert!(String::from_utf8(result.unwrap()).unwrap().contains("ModuleID"));
    }

    #[test]
    fn test_classical_target_debug() {
        let cranelift = ClassicalTarget::Cranelift;
        let llvm = ClassicalTarget::Llvm;
        assert!(format!("{:?}", cranelift).contains("Cranelift"));
        assert!(format!("{:?}", llvm).contains("Llvm"));
    }
}
