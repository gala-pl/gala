# Module 3: No-cloning, and reading the errors

The no-cloning theorem says an unknown quantum state cannot be copied. Gala enforces this with
linear types: a qubit must be used exactly once. The compiler's errors for this rule are the most
useful teaching tool in the language, so this module reads them one at a time. Each sample below
is a real program, and each error code is the code the compiler prints.

The commands you need are `gala check` and `gala explain`:

```
./target/debug/gala check file.gala      # prints the error with its code, exits 1
./target/debug/gala explain E0412        # the long-form explanation for that code
```

## E0401: no cloning

```gala
fn f(q: Qubit) -> (Qubit, Qubit) quantum {
    clone(q)
}
```

```
$ ./target/debug/gala check clone.gala
[E0401] Error: quantum values cannot be cloned (no-cloning theorem)
```

The fix is to create a fresh qubit, not to copy the old one. Gala has no `clone` for qubits.

## E0412: use after consumption

```gala
fn f() -> Qubit quantum {
    let q = qubit()
    let a = q
    let b = q
    b
}
```

```
$ ./target/debug/gala check moved.gala
[E0412] Error: qubit `q` used after it was consumed
```

`let a = q` moves `q` into `a`. The second `let` uses the name `q`, which no longer refers to a
live qubit. A qubit is consumed by a binding that moves it, by `measure`, and by `drop`.

## E0403: the same qubit passed twice

```gala
fn f() quantum {
    let a = qubit()
    cx(a, a)
    drop(a)
}
```

```
$ ./target/debug/gala check alias.gala
[E0403] Error: qubit `a` is passed twice in one call
```

A two-qubit gate needs two distinct qubits. Allocate the second one with `qubit()`.

## E0402: implicit drop

```gala
fn f() quantum {
    let q = qubit()
    h(q);
}
```

```
$ ./target/debug/gala check implicit.gala
[E0402] Error: qubit `q` is never consumed
```

Reaching the end of a scope with a live qubit is an error. Release it with `drop(q)`, or measure
it, or return it.

## E0405: consumed on only some branches

```gala
fn f(c: Bool) quantum {
    let q = qubit()
    if c {
        drop(q)
    }
}
```

```
$ ./target/debug/gala check branch.gala
[E0405] Error: qubit `q` is consumed on only some branches
```

After the `if`, `q` is either live or gone, depending on `c`, and the checker cannot tell which.
Consume it on both paths:

```gala
fn f(c: Bool) quantum {
    let q = qubit()
    if c {
        drop(q)
    } else {
        drop(q)
    }
}
```

This version is accepted.

## Exercise

Write a function that allocates two qubits, applies `cx` to them, and releases both. Run
`gala check`. Then remove one `drop` and find which code is reported. Run
`./target/debug/gala explain` on that code and read the explanation.
