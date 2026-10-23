# Module 1: Hello, classical Gala

Gala is a language with two worlds. Classical code is ordinary: integers, booleans, floats, and
functions. Quantum code manipulates qubits. The compiler keeps the two apart by effects, which
you will meet in module 4. This module is entirely classical.

## A program

```gala
fn greet(name: Int) -> Int pure {
    print(name)
    name * 2
}

fn main() -> Int {
    let answer = greet(21)
    print(answer)
    0
}
```

Save it as `hello.gala`, then:

```
./target/debug/gala check hello.gala     # ok: hello.gala checks passed
./target/debug/gala run hello.gala       # prints 21, then 42, then result: 0
```

## What to notice

- `fn name(params) -> Type effect { body }` declares a function. The last expression of the body
  is its value. `return` is also available.
- `let` binds a name. Bindings are immutable unless written `let mut`, and assigning to an
  immutable binding is error E0203.
- `greet` is declared `pure`. A `pure` function may call only other `pure` functions and `print`.
  Declaring the effect is a claim the compiler checks. If the body needed a qubit, the claim
  would be refused (module 4).
- Gala does not convert between `Int`, `Float`, and `Bool`. Passing a `Bool` where an `Int` is
  expected is error E0201 (type mismatch).

## Try it

Change `greet(21)` to `greet(true)` and run `gala check`. You get E0201. Then change it back.
