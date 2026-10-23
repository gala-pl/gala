//! Differentiation of hybrid programs by the parameter-shift rule.
//!
//! The quantity differentiated is the expectation `E(θ) = P(false) − P(true)` of a function that
//! returns `Measured<Bool>` (the Pauli-Z expectation of its measured qubit). For a rotation gate
//! `R(θ)` with generator `G/2`, `∂E/∂θ = (E(θ+π/2) − E(θ−π/2)) / 2` exactly, and this holds per
//! gate occurrence. The expectation is estimated from shots, so gradients carry sampling noise
//! that shrinks with the shot count.
//!
//! Supported: a parameter that reaches rotation gates (`rx`, `ry`, `rz`) directly, as the gate's
//! angle, through copies. A parameter that is first combined arithmetically is rejected, because
//! the chain rule through that expression is not implemented yet.

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{GateKind, Gir, Inst, Slot};
use gala_sim::{run_function, ShiftPlan, SimConfig, Value};
use std::collections::HashSet;
use std::f64::consts::FRAC_PI_2;

/// A gradient and the cost of computing it.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// Estimated `∂E/∂θ`.
    pub value: f64,
    /// Number of rotation gates whose angle depends on the parameter.
    pub occurrences: usize,
    /// Device submissions used (one per shifted circuit evaluation).
    pub submissions: u64,
}

/// Estimate `E` at `args`, with an optional shift plan. Returns the estimate and the number of
/// submissions it used (one).
fn expectation(
    gir: &Gir,
    func: usize,
    args: &[Value],
    cfg: &SimConfig,
    shifts: &ShiftPlan,
) -> Result<f64, Diagnostics> {
    let report =
        run_function(gir, func, args.to_vec(), cfg, shifts).map_err(|e| diag(e.code, e.message))?;
    let shots = report.shots.max(1) as f64;
    let count = |key: &str| report.histogram.get(key).copied().unwrap_or(0) as f64;
    Ok((count("false") - count("true")) / shots)
}

fn diag(code: gala_diagnostics::DiagnosticCode, msg: impl Into<String>) -> Diagnostics {
    let mut d = Diagnostics::new();
    d.push(Diagnostic::error(code, msg));
    d
}

/// A rotation gate whose angle depends on the differentiated parameter: `(function, block, index)`.
type Site = (usize, u32, usize);

/// Rotation gates whose angle depends on `param_slot` of `func`, following the parameter through
/// copies and through calls to other functions (the callee's matching parameter is followed in
/// turn). Arithmetic on the parameter is rejected, because its chain rule is not implemented.
fn occurrences(gir: &Gir, func: usize, param_slot: Slot) -> Result<Vec<Site>, Diagnostics> {
    let mut out = Vec::new();
    let mut seen: HashSet<(usize, Slot)> = HashSet::new();
    collect_sites(gir, func, param_slot, &mut seen, &mut out, 0)?;
    Ok(out)
}

fn collect_sites(
    gir: &Gir,
    func: usize,
    param_slot: Slot,
    seen: &mut HashSet<(usize, Slot)>,
    out: &mut Vec<Site>,
    depth: usize,
) -> Result<(), Diagnostics> {
    if depth > 32 || !seen.insert((func, param_slot)) {
        return Ok(());
    }
    let f = &gir.funcs[func];
    let mut dep: HashSet<Slot> = HashSet::from([param_slot]);
    let mut arithmetic: HashSet<Slot> = HashSet::new();
    for b in &f.blocks {
        for inst in &b.insts {
            match inst {
                Inst::Copy { dst, src } if dep.contains(src) => {
                    dep.insert(*dst);
                }
                Inst::Bin { dst, a, b, .. } if dep.contains(a) || dep.contains(b) => {
                    arithmetic.insert(*dst);
                    dep.insert(*dst);
                }
                Inst::Un { dst, src, .. } if dep.contains(src) => {
                    arithmetic.insert(*dst);
                    dep.insert(*dst);
                }
                _ => {}
            }
        }
    }
    for (bi, b) in f.blocks.iter().enumerate() {
        for (ii, inst) in b.insts.iter().enumerate() {
            match inst {
                Inst::Gate { gate, angle: Some(a), .. } if dep.contains(a) => {
                    if arithmetic.contains(a) {
                        return Err(diag(
                            codes::UNSUPPORTED_FEATURE,
                            "the parameter is combined arithmetically before reaching a gate; only direct use is differentiated",
                        ));
                    }
                    if !matches!(gate, GateKind::Rx | GateKind::Ry | GateKind::Rz) {
                        return Err(diag(
                            codes::UNSUPPORTED_FEATURE,
                            format!("parameter-shift is implemented for rx, ry, rz; `{}` uses a different rule", gate.name()),
                        ));
                    }
                    out.push((func, bi as u32, ii));
                }
                Inst::Call { func: callee, args, .. } => {
                    for (i, a) in args.iter().enumerate() {
                        if dep.contains(a) {
                            if let Some(&callee_param) = gir.funcs[*callee].params.get(i) {
                                collect_sites(gir, *callee, callee_param, seen, out, depth + 1)?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Gradient of `E` with respect to argument `param` of function `func`, at `args`.
pub fn gradient(
    gir: &Gir,
    func: usize,
    param: usize,
    args: &[Value],
    cfg: &SimConfig,
) -> Result<Gradient, Diagnostics> {
    let f = &gir.funcs[func];
    let param_slot = *f.params.get(param).ok_or_else(|| {
        diag(codes::RUNTIME_ERROR, format!("`{}` has no parameter #{param}", f.name))
    })?;
    let sites = occurrences(gir, func, param_slot)?;
    let mut total = 0.0;
    let mut submissions = 0u64;
    for site in &sites {
        let mut plus = ShiftPlan::new();
        plus.insert(*site, FRAC_PI_2);
        let mut minus = ShiftPlan::new();
        minus.insert(*site, -FRAC_PI_2);
        let e_plus = expectation(gir, func, args, cfg, &plus)?;
        let e_minus = expectation(gir, func, args, cfg, &minus)?;
        submissions += 2;
        total += (e_plus - e_minus) / 2.0;
    }
    Ok(Gradient { value: total, occurrences: sites.len(), submissions })
}

/// Central finite difference of `E` in argument `param`, with step `h`. Uses the same seed for
/// both evaluations, so sampling noise is correlated and largely cancels.
pub fn finite_difference(
    gir: &Gir,
    func: usize,
    param: usize,
    args: &[Value],
    cfg: &SimConfig,
    h: f64,
) -> Result<f64, Diagnostics> {
    let shift = |delta: f64| -> Vec<Value> {
        args.iter()
            .enumerate()
            .map(|(i, v)| match (i == param, v) {
                (true, Value::Float(x)) => Value::Float(x + delta),
                (true, Value::Int(x)) => Value::Float(*x as f64 + delta),
                _ => v.clone(),
            })
            .collect()
    };
    let plus = expectation(gir, func, &shift(h), cfg, &ShiftPlan::new())?;
    let minus = expectation(gir, func, &shift(-h), cfg, &ShiftPlan::new())?;
    Ok((plus - minus) / (2.0 * h))
}

/// One labelled training example: the non-parameter arguments and the expected bit.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Arguments to the classifier, with a placeholder at the trainable parameter's position.
    pub args: Vec<Value>,
    /// Target label.
    pub label: bool,
}

/// Result of training a classifier.
#[derive(Debug, Clone)]
pub struct TrainReport {
    /// Learned parameter.
    pub theta: f64,
    /// Fraction of samples classified correctly at the final parameter.
    pub accuracy: f64,
    /// Loss at each step.
    pub loss_history: Vec<f64>,
    /// Total device submissions used.
    pub submissions: u64,
}

/// Hyperparameters for [`train`].
#[derive(Debug, Clone)]
pub struct TrainOptions {
    /// Starting value of the parameter.
    pub init: f64,
    /// Gradient-descent steps.
    pub steps: usize,
    /// Step size.
    pub learning_rate: f64,
}

/// Train one parameter by gradient descent on the squared error between `P(true)` and the label.
///
/// `P(true) = (1 − E) / 2`, so `∂L/∂θ = mean 2(P − y) · (−∂E/∂θ / 2)`.
pub fn train(
    gir: &Gir,
    func: usize,
    param: usize,
    samples: &[Sample],
    opts: &TrainOptions,
    cfg: &SimConfig,
) -> Result<TrainReport, Diagnostics> {
    let mut theta = opts.init;
    let steps = opts.steps;
    let learning_rate = opts.learning_rate;
    let mut loss_history = Vec::with_capacity(steps);
    let mut submissions = 0u64;
    for _ in 0..steps {
        let mut grad = 0.0;
        let mut loss = 0.0;
        for (k, s) in samples.iter().enumerate() {
            let mut args = s.args.clone();
            args[param] = Value::Float(theta);
            let g = gradient(
                gir,
                func,
                param,
                &args,
                &SimConfig { seed: cfg.seed.wrapping_add(k as u64), ..cfg.clone() },
            )?;
            submissions += g.submissions + 1;
            let e = expectation(
                gir,
                func,
                &args,
                &SimConfig { seed: cfg.seed.wrapping_add(k as u64), ..cfg.clone() },
                &ShiftPlan::new(),
            )?;
            let p = (1.0 - e) / 2.0;
            let y = if s.label { 1.0 } else { 0.0 };
            loss += (p - y) * (p - y);
            grad += 2.0 * (p - y) * (-g.value / 2.0);
        }
        let n = samples.len().max(1) as f64;
        loss_history.push(loss / n);
        theta -= learning_rate * grad / n;
    }
    // Accuracy at the learned parameter, judged by the sign of the expectation.
    let mut correct = 0usize;
    for (k, s) in samples.iter().enumerate() {
        let mut args = s.args.clone();
        args[param] = Value::Float(theta);
        let e = expectation(
            gir,
            func,
            &args,
            &SimConfig { seed: cfg.seed.wrapping_add(1000 + k as u64), ..cfg.clone() },
            &ShiftPlan::new(),
        )?;
        let predicted = e < 0.0;
        if predicted == s.label {
            correct += 1;
        }
    }
    Ok(TrainReport {
        theta,
        accuracy: correct as f64 / samples.len().max(1) as f64,
        loss_history,
        submissions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compile(src: &str) -> Gir {
        let mut sources = gala_driver::Sources::default();
        gala_driver::compile_source(src, &mut sources).expect("compiles").gir
    }

    const CLASSIFIER: &str = "fn classify(x: Float, theta: Float) -> Measured<Bool> prob {\n    let q = qubit()\n    ry(q, x)\n    ry(q, theta)\n    measure(q)\n}\nfn main() -> Int { 0 }\n";

    #[test]
    fn parameter_shift_matches_finite_difference() {
        let gir = compile(CLASSIFIER);
        let func = gir.func_named("classify").unwrap();
        let args = vec![Value::Float(0.4), Value::Float(0.9)];
        let cfg = SimConfig { shots: 40_000, seed: 11, ..Default::default() };
        let g = gradient(&gir, func, 1, &args, &cfg).unwrap();
        let fd = finite_difference(&gir, func, 1, &args, &cfg, 0.05).unwrap();
        assert_eq!(g.occurrences, 1);
        assert!((g.value - fd).abs() < 0.05, "shift {} vs finite difference {}", g.value, fd);
    }

    #[test]
    fn analytic_gradient_of_single_ry() {
        // E(θ) = cos(x + θ) for Ry(x) then Ry(θ) on |0⟩ measured in Z, so ∂E/∂θ = −sin(x+θ).
        let gir = compile(CLASSIFIER);
        let func = gir.func_named("classify").unwrap();
        let (x, theta) = (0.4f64, 0.9f64);
        let args = vec![Value::Float(x), Value::Float(theta)];
        let g = gradient(
            &gir,
            func,
            1,
            &args,
            &SimConfig { shots: 60_000, seed: 3, ..Default::default() },
        )
        .unwrap();
        let exact = -(x + theta).sin();
        assert!((g.value - exact).abs() < 0.04, "got {} want {exact}", g.value);
    }

    #[test]
    fn arithmetic_use_is_rejected() {
        let gir = compile("fn c(theta: Float) -> Measured<Bool> prob {\n let q = qubit()\n ry(q, theta * 2.0)\n measure(q)\n}\nfn main() -> Int { 0 }\n");
        let func = gir.func_named("c").unwrap();
        let err = gradient(&gir, func, 0, &[Value::Float(0.1)], &SimConfig::default()).unwrap_err();
        assert!(err.errors().iter().any(|d| d.code.to_string() == "E0207"));
    }

    #[test]
    fn classifier_trains_to_target_accuracy() {
        // Angle encoding of x with a trainable Ry(θ): x = 0 should map to 0 and x = π to 1.
        let gir = compile(CLASSIFIER);
        let func = gir.func_named("classify").unwrap();
        let samples = vec![
            Sample { args: vec![Value::Float(0.0), Value::Float(0.0)], label: false },
            Sample {
                args: vec![Value::Float(std::f64::consts::PI), Value::Float(0.0)],
                label: true,
            },
        ];
        let opts = TrainOptions { init: 0.3, steps: 25, learning_rate: 1.0 };
        let report = train(
            &gir,
            func,
            1,
            &samples,
            &opts,
            &SimConfig { shots: 2_000, seed: 5, ..Default::default() },
        )
        .unwrap();
        assert!(report.accuracy >= 1.0, "accuracy {} theta {}", report.accuracy, report.theta);
        let first = report.loss_history.first().copied().unwrap_or(0.0);
        let last = report.loss_history.last().copied().unwrap_or(0.0);
        assert!(last < first, "loss decreases: {first} -> {last}");
    }
}

/// Gradient computed as one batched device submission.
///
/// The shifted circuits of a gradient are independent, so the device API accepts them together:
/// every circuit is run with the same configuration as in [`gradient`], and the whole batch counts
/// as one submission. The value equals the unbatched gradient exactly.
pub fn gradient_batched(
    gir: &Gir,
    func: usize,
    param: usize,
    args: &[Value],
    cfg: &SimConfig,
) -> Result<Gradient, Diagnostics> {
    let f = &gir.funcs[func];
    let param_slot = *f.params.get(param).ok_or_else(|| {
        diag(codes::RUNTIME_ERROR, format!("`{}` has no parameter #{param}", f.name))
    })?;
    let sites = occurrences(gir, func, param_slot)?;
    // One batch: every plus and minus circuit, in site order.
    let mut batch: Vec<ShiftPlan> = Vec::with_capacity(2 * sites.len());
    for site in &sites {
        let mut plus = ShiftPlan::new();
        plus.insert(*site, FRAC_PI_2);
        let mut minus = ShiftPlan::new();
        minus.insert(*site, -FRAC_PI_2);
        batch.push(plus);
        batch.push(minus);
    }
    let estimates: Vec<f64> = batch
        .iter()
        .map(|plan| expectation(gir, func, args, cfg, plan))
        .collect::<Result<_, _>>()?;
    let total: f64 = estimates.chunks(2).map(|pair| (pair[0] - pair[1]) / 2.0).sum();
    Ok(Gradient { value: total, occurrences: sites.len(), submissions: 1 })
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn batched_gradient_equals_unbatched_in_one_submission() {
        let mut sources = gala_driver::Sources::default();
        let gir = gala_driver::compile_source(
            "fn classify(x: Float, theta: Float) -> Measured<Bool> prob {\n    let q = qubit()\n    ry(q, x)\n    ry(q, theta)\n    measure(q)\n}\nfn main() -> Int { 0 }\n",
            &mut sources,
        )
        .expect("compiles")
        .gir;
        let func = gir.func_named("classify").unwrap();
        let args = vec![Value::Float(0.4), Value::Float(0.9)];
        let cfg = SimConfig { shots: 4_000, seed: 21, ..Default::default() };
        let plain = gradient(&gir, func, 1, &args, &cfg).unwrap();
        let batched = gradient_batched(&gir, func, 1, &args, &cfg).unwrap();
        assert!((plain.value - batched.value).abs() < 1e-12, "same value");
        assert_eq!(batched.submissions, 1, "one submission for the whole gradient");
        assert_eq!(plain.submissions, 2 * plain.occurrences as u64);
    }
}
