# Module 4: Effects, the boundary between classical and quantum

Every function in Gala has an effect: `pure`, `quantum`, or `prob`. The effects form a lattice,
`pure ⊑ quantum ⊑ prob`:

- `pure` code is classical. It may call only other `pure` code and `print`.
- `quantum` code allocates qubits and applies gates, but never measures. Its operations are
  unitary, so it can be reversed.
- `prob` code may measure. Measurement is random, so a `prob` result is a distribution of
  outcomes and not a single value.

A function may call functions whose effect is at or below its own. The effect is part of the
function's signature, and the compiler checks it.

## Accepted: effects that match

```gala
fn prep(q: Qubit) -> Qubit quantum {
    h(q)
}

fn coin() -> Measured<Bool> prob {
    let q = qubit()
    let q = prep(q)
    measure(q)
}

fn main() -> Int prob {
    print(coin())
    0
}
```

`prep` is `quantum` because it applies a gate and does not measure. `coin` is `prob` because it
measures, and it can call the `quantum` function `prep`.

## E0304: measuring in a quantum function

```gala
fn f() -> Measured<Bool> quantum {
    let q = qubit()
    measure(q)
}
```

```
$ ./target/debug/gala check measure_quantum.gala
[E0304] Error: `quantum` function cannot call `measure`, which is `prob`
```

The declaration says "this function is reversible", and `measure` is not reversible. Change the
declaration to `prob`.

## E0303: a quantum function calling a probabilistic one

```gala
fn m(q: Qubit) -> Measured<Bool> prob {
    measure(q)
}

fn f(q: Qubit) -> Measured<Bool> quantum {
    m(q)
}
```

```
$ ./target/debug/gala check call_prob.gala
[E0303] Error: `quantum` function cannot call `m`, which is `prob`
```

The effect of `m` propagates to its callers. Declare `f` as `prob`.

## E0302: a pure function touching qubits

```gala
fn f(x: Int) -> Int pure {
    let q = qubit()
    drop(q)
    x
}
```

```
$ ./target/debug/gala check pure_qubit.gala
[E0302] Error: `pure` function cannot call `qubit`, which is `quantum`
[E0302] Error: `pure` function cannot call `drop`, which is `quantum`
```

Both calls are reported. The fix is to declare `f` as `quantum` (or `prob`, if it measures).

## Why this matters

The effect is the contract at the classical/quantum seam. A `quantum` function can be inverted
and differentiated. A `prob` function cannot be reversed, and asking for its adjoint is a type
error. Reading the effect of a signature tells you which of those operations are available.

## Exercise

Change `coin` from the accepted program to `pure`, and run `gala check`. It reports E0302 for
`qubit` and `prep`, and E0304 for `measure`. Fix the signature by choosing the lowest effect that
the body needs, which is `prob`.
