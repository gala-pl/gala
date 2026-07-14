//! Differentiation lowering for Gala (param-shift + autodiff).

use gala_gir::{Gir, GirFunc};

/// Lower gradient operations in GIR.
pub fn lower_gradients(gir: &mut Gir) {
    for func in gir.funcs.values_mut() {
        lower_grad_in_func(func);
    }
}

fn lower_grad_in_func(func: &mut GirFunc) {
    let _ = func;
}

/// Parameter-shift rule for quantum gates.
pub fn param_shift_gradient(circuit: &GirFunc, param_idx: usize, shift: f64) -> GirFunc {
    let _ = (circuit, param_idx, shift);
    circuit.clone()
}

/// Reverse-mode autodiff for classical code.
pub fn reverse_mode_autodiff(func: &GirFunc, wrt: &[usize]) -> GirFunc {
    let _ = (func, wrt);
    func.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_empty() {
        let mut gir = Gir::default();
        lower_gradients(&mut gir);
    }

    #[test]
    fn test_param_shift_gradient_returns_copy() {
        let mut func = GirFunc::new_test();
        let shifted = param_shift_gradient(&func, 0, 0.5);
        assert_eq!(shifted.name.0, func.name.0);
    }

    #[test]
    fn test_reverse_mode_autodiff_returns_copy() {
        let func = GirFunc::new_test();
        let grads = reverse_mode_autodiff(&func, &[0]);
        assert_eq!(grads.name.0, func.name.0);
    }

    #[test]
    fn test_lower_gradients_no_panic() {
        let mut gir = Gir::default();
        lower_gradients(&mut gir);
    }
}
