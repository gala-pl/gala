//! Backends: the same program runs on any backend that supports it; only the backend line changes.

use gala_driver::{compile_source, Sources};
use gala_sim::{run_entry, SimConfig};

fn program(backend_line: &str) -> String {
    format!(
        "fn flip() -> Measured<Bool> prob {{\n    let q = qubit()\n    x(q)\n    measure(q)\n}}\n\nfn main() -> Int prob {{\n    let backend = {backend_line}\n    let ones = run(flip, on: backend, shots: 4000)\n    print(ones)\n    0\n}}\n"
    )
}

fn ones(backend_line: &str) -> i64 {
    let mut sources = Sources::default();
    let p = compile_source(&program(backend_line), &mut sources).expect("compiles");
    let report =
        run_entry(&p.gir, &SimConfig { shots: 1, seed: 1, ..Default::default() }).expect("runs");
    report.output[0].parse().expect("an integer count")
}

#[test]
fn simulator_measures_every_shot_true() {
    assert_eq!(ones("backend.simulator()"), 4000);
}

#[test]
fn noiseless_backend_matches_the_simulator_exactly() {
    assert_eq!(ones("backend.noisy(0.0)"), ones("backend.simulator()"));
}

#[test]
fn depolarizing_error_flips_back_two_thirds_of_the_error_rate() {
    // X then depolarizing error p: X or Y (probability 2p/3) turns |1⟩ back into |0⟩.
    let expected = 4000.0 * (1.0 - 2.0 * 0.3 / 3.0);
    let got = ones("backend.noisy(0.3)") as f64;
    assert!((got - expected).abs() < 4000.0 * 0.02, "got {got}, expected about {expected}");
}
