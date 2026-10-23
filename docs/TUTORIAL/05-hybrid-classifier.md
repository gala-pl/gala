# Module 5: A hybrid classifier

A variational quantum classifier has three parts: a classical feature, a quantum circuit that
encodes the feature as a rotation, and a trainable rotation whose angle is learned. The
measurement gives a probability, and training adjusts the trainable angle so that the
probabilities match labels. This module builds the smallest version of that loop.

## Part 1: use the library classifier

`library/std/vqa.gala` provides `classify`, a one-qubit classifier:

```gala
import gala.vqa.{ classify }

fn main() -> Int prob {
    print(classify(3.141592653589793, 0.0))
    0
}
```

```
./target/debug/gala check classify.gala   # ok: classify.gala checks passed
./target/debug/gala run classify.gala     # prints true, then result: 0
```

`classify(x, theta)` encodes `x` as a rotation of |0⟩, applies a second rotation by `theta`, and
measures. With `x = π` and `theta = 0` the qubit is flipped to |1⟩, so the output is `true`. With
`x = 0` it is `false`. The trainable parameter is `theta`.

## Part 2: write your own classifier

The library's `classify` calls a helper function for its ansatz. The gradient code in `gala-diff`
looks for the trainable parameter where it reaches a rotation gate directly, so for training we
write the rotations inline:

```gala
fn classify(x: Float, theta: Float) -> Measured<Bool> prob {
    let q = qubit()
    ry(q, x)
    ry(q, theta)
    measure(q)
}

fn main() -> Int prob {
    print(classify(0.0, 0.0))
    0
}
```

`gala check` accepts this file, and `gala run` prints `false`.

## Part 3: train it in Rust

Training is a Rust library call, not a Gala feature. The Gala source above is compiled to GIR, and
`gala-diff` estimates the gradient of the expected value with the parameter-shift rule, then runs
gradient descent. The call below follows the crate's own test, `classifier_trains_to_target_accuracy`:

```rust
use gala_diff::{train, Sample, TrainOptions};
use gala_sim::{SimConfig, Value};
use std::f64::consts::PI;

let mut sources = gala_driver::Sources::default();
let gir = gala_driver::compile_source(CLASSIFIER_SRC, &mut sources)
    .expect("compiles")
    .gir;
let func = gir.func_named("classify").unwrap();

// Feature x = 0 should be labelled false, and x = pi should be labelled true.
let samples = vec![
    Sample { args: vec![Value::Float(0.0), Value::Float(0.0)], label: false },
    Sample { args: vec![Value::Float(PI), Value::Float(0.0)], label: true },
];
let opts = TrainOptions { init: 0.3, steps: 25, learning_rate: 1.0 };
let cfg = SimConfig { shots: 2_000, seed: 5, ..Default::default() };

// Parameter index 1 is `theta`.
let report = train(&gir, func, 1, &samples, &opts, &cfg).unwrap();
println!("theta = {:.3}, accuracy = {:.2}", report.theta, report.accuracy);
```

Here `CLASSIFIER_SRC` is the Gala source from Part 2. The `gala-diff` test
`classifier_trains_to_target_accuracy` uses the same classifier with a placeholder `main`, and it
passes (checked with `cargo test -p gala-diff`). The report gives the
learned `theta`, the accuracy on the samples, the loss at each step, and the number of device
submissions used.

## What to notice

- The Gala program describes the circuit and the measurement. It does not describe the training
  loop, which is ordinary Rust around the compiled circuit.
- The gradient is exact for rotation angles that reach gates directly. An angle that is first
  combined arithmetically, such as `theta * 2.0`, is rejected: the gradient code reports E0207
  (feature not supported yet). Try `grad` in Gala as well: it is also E0207 today.
- Gradients are estimated from shots, so accuracy and loss carry sampling noise.

## Exercise

Change `init` to `0.0` and watch how many steps are needed. Then increase `shots` and compare the
loss history.
