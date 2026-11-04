//! Property tests for standard library algorithms: verifies unitarity/reversibility of
//! all quantum functions in gala.algorithms.

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

#[test]
fn qft2_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft2, iqft2 }

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "qft2").expect("qft2 found");
    for input in 0..4 {
        let fidelity = reversible_roundtrip(&gir, func, 2, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "qft2 input {input:02b}: fidelity {fidelity}");
    }
}

#[test]
fn iqft2_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft2, iqft2 }

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "iqft2").expect("iqft2 found");
    for input in 0..4 {
        let fidelity = reversible_roundtrip(&gir, func, 2, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "iqft2 input {input:02b}: fidelity {fidelity}");
    }
}

#[test]
fn qft3_is_unitary() {
    // Instantiate qft<3> by calling it
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft, iqft }

fn test_qft3(q: Qubits<3>) -> Qubits<3> quantum {
    qft<3>(q)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_qft3").expect("test_qft3 found");
    for input in 0..8 {
        let fidelity = reversible_roundtrip(&gir, func, 3, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "qft<3> input {input:03b}: fidelity {fidelity}");
    }
}

#[test]
fn iqft3_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft, iqft }

fn test_iqft3(q: Qubits<3>) -> Qubits<3> quantum {
    iqft<3>(q)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_iqft3").expect("test_iqft3 found");
    for input in 0..8 {
        let fidelity = reversible_roundtrip(&gir, func, 3, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "iqft<3> input {input:03b}: fidelity {fidelity}");
    }
}

#[test]
fn qft4_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft, iqft }

fn test_qft4(q: Qubits<4>) -> Qubits<4> quantum {
    qft<4>(q)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_qft4").expect("test_qft4 found");
    for input in 0..16 {
        let fidelity = reversible_roundtrip(&gir, func, 4, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "qft<4> input {input:04b}: fidelity {fidelity}");
    }
}

#[test]
fn iqft4_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ qft, iqft }

fn test_iqft4(q: Qubits<4>) -> Qubits<4> quantum {
    iqft<4>(q)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_iqft4").expect("test_iqft4 found");
    for input in 0..16 {
        let fidelity = reversible_roundtrip(&gir, func, 4, input, true).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "iqft<4> input {input:04b}: fidelity {fidelity}");
    }
}

#[test]
#[ignore = "reversible_roundtrip doesn't support multi-qubit args yet"]
fn toffoli_is_unitary() {
    // toffoli takes 3 individual qubits - test infrastructure doesn't support this yet
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ toffoli }

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "toffoli").expect("toffoli found");
    for input in 0..8 {
        let fidelity = reversible_roundtrip(&gir, func, 3, input, false).unwrap();
        assert!((fidelity - 1.0).abs() < 1e-10, "toffoli input {input:03b}: fidelity {fidelity}");
    }
}

#[test]
#[ignore = "reversible_roundtrip doesn't support multi-qubit args yet"]
fn full_adder_is_unitary() {
    // full_adder takes 4 individual qubits - test infrastructure doesn't support this yet
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ full_adder }

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "full_adder").expect("full_adder found");
    for input in 0..16 {
        let fidelity = reversible_roundtrip(&gir, func, 4, input, false).unwrap();
        assert!(
            (fidelity - 1.0).abs() < 1e-10,
            "full_adder input {input:04b}: fidelity {fidelity}"
        );
    }
}

#[test]
fn trotter_step2_is_unitary() {
    let src = r#"
import gala.gates.*
import gala.core.*
import gala.algorithms.{ trotter_step2 }

// Wrap trotter_step2 with a fixed dt parameter
fn test_trotter(q: Qubits<2>) -> Qubits<2> quantum {
    trotter_step2(q, 0.5)
}

fn main() -> Int { 0 }
"#;
    let gir = compile_gir(src);
    let func = find_func(&gir, "test_trotter").expect("test_trotter found");
    for input in 0..4 {
        let fidelity = reversible_roundtrip(&gir, func, 2, input, true).unwrap();
        assert!(
            (fidelity - 1.0).abs() < 1e-10,
            "trotter_step2 input {input:02b}: fidelity {fidelity}"
        );
    }
}
