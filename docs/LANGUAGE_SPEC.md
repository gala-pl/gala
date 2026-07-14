# 01 — Language Specification

This is the surface-language reference: lexical structure, grammar, type notation, effects, and
worked examples. Semantics of the type system are detailed in
[TYPE_SYSTEM.md](./TYPE_SYSTEM.md).

## 1. Design of the surface syntax

Gala's syntax is deliberately familiar — expression-oriented, braces, `fn`, `let`, type
inference — so a Python/Rust/Swift engineer is productive on day one. The quantum-specific
machinery lives in **type annotations and effects**, not in exotic syntax. Everything is an
expression where reasonable; blocks return their trailing expression.

## 2. Lexical structure

- **Identifiers:** `[A-Za-z_][A-Za-z0-9_]*`. Greek letters are permitted in identifiers
  (`θ`, `φ`, `ψ`) as a nod to the domain; ASCII aliases (`theta`) always exist.
- **Comments:** `//` line, `/* ... */` block (nestable), `///` doc comment.
- **Literals:** integers (`42`, `0xFF`, `0b1010`), floats (`3.14`, `1e-3`), complex
  (`2+3i`), bool (`true`/`false`), strings (`"..."`), unit (`()`).
- **Keywords:** `fn let mut if else match for in while return import type struct enum trait
  impl pure quantum prob const true false`.
- **Reserved for quantum:** `qubit qubits measure reverse adjoint control grad drop`.

## 3. Grammar (EBNF, non-normative)

```
program      := item*
item         := fn_def | struct_def | enum_def | trait_def | impl_block | import | type_alias | const_def

fn_def       := "fn" ident generics? "(" params? ")" ("->" type)? effect? where_clause? block
generics     := "<" generic_param ("," generic_param)* ">"
generic_param:= ident (":" bound)? | "const" ident ":" type
effect       := "pure" | "quantum" | "prob" | effect_var
params       := param ("," param)*
param        := "mut"? pattern ":" type
where_clause := "where" (bound ("," bound)*)

type         := path type_args?
              | "Qubits" "<" const_expr ">"
              | "Qubit"
              | "Measured" "<" type ">"
              | "(" (type ("," type)*)? ")"          // tuple / unit
              | "[" type ";" const_expr "]"          // array
              | "&" type                              // borrow (classical only)
              | fn_type
fn_type      := "fn" "(" (type ("," type)*)? ")" ("->" type)? effect?
type_args    := "<" (type | const_expr) ("," (type | const_expr))* ">"

block        := "{" stmt* expr? "}"
stmt         := let_stmt | expr ";" | item
let_stmt     := "let" "mut"? pattern (":" type)? "=" expr ";"

expr         := literal | path | call | method_call | binop | unop
              | if_expr | match_expr | for_expr | while_expr | block | tuple | array | lambda
call         := path "(" args? ")"
method_call  := expr "." ident "(" args? ")"
lambda       := "|" params? "|" ("->" type)? effect? (expr | block)
if_expr      := "if" expr block ("else" (if_expr | block))?
match_expr   := "match" expr "{" (pattern "=>" expr ",")* "}"
for_expr     := "for" pattern "in" expr block
args         := (arg ("," arg)*)
arg          := (ident ":")? expr                     // optional named args
```

## 4. Types

| Type | Meaning | Copy? |
|------|---------|-------|
| `Qubit` | A single linear qubit resource | **No** (linear) |
| `Qubits<N>` | A register of `N` qubits; `N` is a const generic | **No** (linear) |
| `Measured<T>` | A classical `T` produced by measurement, carrying its distribution | Yes |
| `Bool`, `Int`, `Float`, `Complex` | Classical scalars | Yes |
| `Vec<T>`, `[T; N]`, tuples, `struct`, `enum` | Classical composites | If `T` is |
| `Params` | Trainable parameter bundle for variational circuits | Yes |
| `fn(...) -> T eff` | Function type carrying an effect | Yes |

Linear types (`Qubit`, `Qubits<N>`, and any struct containing them) may not be duplicated,
implicitly dropped, or used after being consumed. See [TYPE_SYSTEM.md](./TYPE_SYSTEM.md).

## 5. Effects

Every function has one of three effect rows (inferred if omitted):

- `pure` — deterministic classical computation.
- `quantum` — unitary evolution of quantum state; reversible; **no measurement**.
- `prob` — has performed measurement; carries classical randomness.

Effect ordering: `pure ⊑ quantum ⊑ prob` for the purpose of "may call." A `quantum` function may
call `pure` functions; calling a `prob` function from a `quantum` context is a type error (you
have crossed the measurement boundary and must acknowledge it by being `prob` yourself). Effect
polymorphism is available via an effect variable in generics (advanced).

## 6. Core operations

```gala
qubit() -> Qubit quantum                 // allocate |0>
qubits<N>() -> Qubits<N> quantum         // allocate N x |0>
h(q: Qubit) -> Qubit quantum             // Hadamard, in place (consumes+returns q)
x, y, z, s, t (q: Qubit) -> Qubit quantum
rx, ry, rz (q: Qubit, θ: Float) -> Qubit quantum
cx(control: Qubit, target: Qubit) -> (Qubit, Qubit) quantum   // CNOT
measure(q: Qubit) -> Measured<Bool> prob
drop(q: Qubit) quantum                   // safe release; compiler synthesizes uncompute
reverse(f) / adjoint(f)                  // compiler-derived inverse of a reversible fn
control(f)                               // compiler-derived controlled version
grad(f, wrt: p)                          // native gradient operator
```

## 7. Worked examples

### 7.1 Bell pair

```gala
fn bell() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    let (a, b) = cx(a, b)   // ownership threaded explicitly, or sugar (see 7.4)
    (a, b)
}
```

### 7.2 A fair coin (crossing the boundary)

```gala
fn coin() -> Measured<Bool> prob {
    let q = qubit()
    h(q)
    measure(q)              // consumes q; function is `prob`
}
```

### 7.3 Variational classifier (hybrid, differentiable)

```gala
import gala.vqa.{ angle_encode, Params }

fn encode(x: Vec<Float>) -> Qubits<4> quantum {
    let q = qubits<4>()
    angle_encode(q, x)
    q
}

fn ansatz(q: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    for layer in 0..θ.depth {
        for i in 0..4 { ry(q[i], θ[layer][i]) }
        for i in 0..3 { cx(q[i], q[i+1]) }
    }
    q
}

fn classify(x: Vec<Float>, θ: Params) -> Measured<Bool> prob {
    let q = ansatz(encode(x), θ)
    measure(q[0])
}

let dθ = grad(classify, wrt: θ)   // parameter-shift synthesized by compiler
```

### 7.4 In-place sugar

Threading ownership by hand is verbose, so gates that take and return the same qubit(s) may be
written as statements; the compiler rebinds the name:

```gala
fn bell() -> Qubits<2> quantum {
    let a = qubit();  let b = qubit()
    h(a)                    // sugar for: let a = h(a)
    cx(a, b)                // sugar for: let (a, b) = cx(a, b)
    (a, b)
}
```

The desugared, explicit form is always valid and is what the checker reasons about.

## 8. Modules & imports

```gala
import gala.core.*
import gala.vqa.{ angle_encode, layered_ansatz }
import mypkg.foo as f
```

A package is a directory with a `gala.toml` manifest. Module paths mirror directory structure.
See [TOOLCHAIN_DX.md](./TOOLCHAIN_DX.md).

## 9. Open syntactic questions

- Whether to make the in-place sugar (7.4) the default and require explicit threading only in
  ambiguous cases.
- Whether `Measured<T>` should auto-coerce to `T` in classical contexts or always require an
  explicit `.value()` / `.sample()`.
- Exact surface for controlled/adjoint application (`control(f)(...)` vs a `ctrl { ... }` block).

These are tracked in [ROADMAP.md](./ROADMAP.md) under "language RFCs."
