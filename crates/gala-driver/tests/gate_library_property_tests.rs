//! Property tests for the standard gate library: verifies unitarity of all builtin gates.

use gala_driver::{compile_source, Sources};
use gala_sim::reversible_roundtrip;

/// Compile a program and return the GIR.
fn compile_gir(src: &str) -> gala_gir::Gir {
    let mut sources = Sources::default();
    let program = compile_source(src, &mut sources).expect("compiles");
    program.gir
}

/// Find a function by name in the GIR.
fn find_func(gir: &gala_gir::Gir, name: &str) -> Option<usize> {
    gir.funcs.iter().position(|f| f.name == name)
}

macro_rules! test_gate_unitary {
    ($name:ident, $gate:expr, $test_name:expr) => {
        #[test]
        fn $name() {
            let src = format!(
                r#"
import gala.gates.*
import gala.core.*

fn test_gate(q: Qubit) -> Qubit quantum {{
    {}(q)
}}

fn main() -> Int {{ 0 }}
"#,
                $gate
            );
            let gir = compile_gir(&src);
            let func = find_func(&gir, "test_gate").expect($test_name);
            for input in 0..2 {
                let fidelity = reversible_roundtrip(&gir, func, 1, input, false).unwrap();
                assert!(
                    (fidelity - 1.0).abs() < 1e-10,
                    "{} input {}: fidelity {}",
                    $test_name,
                    input,
                    fidelity
                );
            }
        }
    };
}

test_gate_unitary!(h_gate_unitary, "h", "h");
test_gate_unitary!(x_gate_unitary, "x", "x");
test_gate_unitary!(y_gate_unitary, "y", "y");
test_gate_unitary!(z_gate_unitary, "z", "z");
test_gate_unitary!(s_gate_unitary, "s", "s");
test_gate_unitary!(t_gate_unitary, "t", "t");

#[test]
fn rx_gate_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*

fn test_rx(q: Qubit) -> Qubit quantum {
    rx(q, 0.5)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_rx").expect("test_rx");
    for input in 0..2 {
        let fidelity = reversible_roundtrip(&gir, func, 1, input, false).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "rx input {}: fidelity {}", input, fidelity);
    }
}

#[test]
fn ry_gate_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*

fn test_ry(q: Qubit) -> Qubit quantum {
    ry(q, 0.5)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_ry").expect("test_ry");
    for input in 0..2 {
        let fidelity = reversible_roundtrip(&gir, func, 1, input, false).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "ry input {}: fidelity {}", input, fidelity);
    }
}

#[test]
fn rz_gate_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*

fn test_rz(q: Qubit) -> Qubit quantum {
    rz(q, 0.5)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_rz").expect("test_rz");
    for input in 0..2 {
        let fidelity = reversible_roundtrip(&gir, func, 1, input, false).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "rz input {}: fidelity {}", input, fidelity);
    }
}

macro_rules! test_two_qubit_gate_unitary {
    ($name:ident, $gate:expr, $test_name:expr) => {
        #[test]
        fn $name() {
            let src = format!(
                r#"
import gala.gates.*
import gala.core.*

fn test_gate(q: Qubits<2>) -> Qubits<2> quantum {{
    {}(q[0], q[1])
    q
}}

fn main() -> Int {{ 0 }}
"#,
                $gate
            );
            let gir = compile_gir(&src);
            let func = find_func(&gir, "test_gate").expect($test_name);
            for input in 0..4 {
                let fidelity = reversible_roundtrip(&gir, func, 2, input, true).unwrap();
                assert!(
                    (fidelity - 1.0).abs() < 1e-10,
                    "{} input {:02b}: fidelity {}",
                    $test_name,
                    input,
                    fidelity
                );
            }
        }
    };
}

test_two_qubit_gate_unitary!(cx_gate_unitary, "cx", "cx");
test_two_qubit_gate_unitary!(cz_gate_unitary, "cz", "cz");
test_two_qubit_gate_unitary!(swap_gate_unitary, "swap", "swap");

#[test]
fn cp_gate_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*

fn test_cp(q: Qubits<2>) -> Qubits<2> quantum {
    cp(q[0], q[1], 0.5)
    q
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_cp").expect("test_cp");
    for input in 0..4 {
        let fidelity = reversible_roundtrip(&gir, func, 2, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "cp input {:02b}: fidelity {}", input, fidelity);
    }
}
