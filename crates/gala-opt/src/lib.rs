//! GIR optimization passes for Gala.

use gala_gir::{Gir, GirFunc};

/// Run all optimization passes on GIR.
pub fn optimize_gir(gir: &mut Gir) {
    for func in gir.funcs.values_mut() {
        optimize_func(func);
    }
}

fn optimize_func(func: &mut GirFunc) {
    fuse_single_qubit_gates(func);
    cancel_adjacent_gates(func);
    merge_rotations(func);
    dead_code_elimination(func);
    constant_folding(func);
}

fn fuse_single_qubit_gates(func: &mut GirFunc) { let _ = func; }
fn cancel_adjacent_gates(func: &mut GirFunc) { let _ = func; }
fn merge_rotations(func: &mut GirFunc) { let _ = func; }
fn dead_code_elimination(func: &mut GirFunc) { let _ = func; }
fn constant_folding(func: &mut GirFunc) { let _ = func; }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_empty() {
        let mut gir = Gir::default();
        optimize_gir(&mut gir);
    }

    #[test]
    fn test_optimize_with_func() {
        let mut gir = Gir::default();
        let func = GirFunc::new_test();
        gir.funcs.insert(func.id, func);
        optimize_gir(&mut gir);
        assert_eq!(gir.funcs.len(), 1);
    }

    #[test]
    fn test_dce_no_panic_on_empty() {
        let mut func = GirFunc::new_test();
        dead_code_elimination(&mut func);
        assert_eq!(func.name.0, "test");
    }

    #[test]
    fn test_constant_folding_no_panic() {
        let mut func = GirFunc::new_test();
        constant_folding(&mut func);
        assert_eq!(func.blocks.len(), 1);
    }

    #[test]
    fn test_fuse_no_panic() {
        let mut func = GirFunc::new_test();
        fuse_single_qubit_gates(&mut func);
    }

    #[test]
    fn test_cancel_no_panic() {
        let mut func = GirFunc::new_test();
        cancel_adjacent_gates(&mut func);
    }
}