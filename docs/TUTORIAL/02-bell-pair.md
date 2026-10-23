# Module 2: A Bell pair

A qubit is a linear resource. You allocate it, transform it with gates, and consume it, either by
measuring it or by releasing it with `drop`. This module builds an entangled pair and measures
one half.

## A program

```gala
fn bell() -> Measured<Bool> prob {
    let a = qubit()
    let b = qubit()
    h(a)
    let (a, b) = cx(a, b)
    drop(b)
    measure(a)
}

fn main() -> Int prob {
    print(bell())
    0
}
```

```
./target/debug/gala check bell.gala      # ok: bell.gala checks passed
./target/debug/gala run bell.gala        # prints true or false, then result: 0
```

The outcome of `measure(a)` is random: each bit has probability one half. The simulator is
seeded, and the six `gala run` calls we made printed `false` every time.

## What to notice

- `qubit()` allocates one qubit in |0⟩. Its effect is `quantum`, so the enclosing function must
  be `quantum` or `prob`.
- `h(a);` is a gate applied as a statement. It acts on `a` in place: the name `a` still refers to
  the qubit afterward, so it can be used again. The qubit is still live and must be consumed
  later. If it were never consumed, the checker would report E0402.
- `cx(a, b)` returns both qubits, so `let (a, b) = cx(a, b)` moves them into new bindings with
  the same names.
- `drop(b)` releases `b`. The checker first confirms that the gates which entangled `b` with `a`
  can be undone while `a` is still live (the uncomputation rule, E0530). Here they can, so
  `b` is released in |0⟩.
- `measure(a)` consumes `a` and returns a `Measured<Bool>`. Measurement is probabilistic, so the
  function is `prob`, the top of the effect lattice.

## Try it

Delete the `drop(b)` line and run `gala check`. The checker reports E0402 for `b`, because a
qubit that goes out of scope unconsumed is an error. Put the line back.
