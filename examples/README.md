# Gala Examples

A collection of runnable Gala programs demonstrating classical, quantum, and hybrid
quantum-classical programming. Each example is a standalone `.gala` file you can
check and run with the Gala compiler:

```sh
cargo build --bin gala
./target/debug/gala check examples/<name>.gala   # type, effect, linearity check
./target/debug/gala run  examples/<name>.gala    # run on the built-in simulator
```

## Classical

| Example | What it demonstrates |
|---------|---------------------|
| [`hello_world.gala`](./hello_world.gala) | Your first Gala program — `fn main()`, `print`, return values |
| [`fib.gala`](./fib.gala) | Recursive function, integer arithmetic |
| [`classical_functions.gala`](./classical_functions.gala) | Pure functions, multiple parameters, boolean logic |
| [`control_flow.gala`](./control_flow.gala) | `if`/`else`, `match`, `for`, `while` — all expressions |
| [`effects.gala`](./effects.gala) | `pure`, `quantum`, `prob` effect system |

## Quantum

| Example | What it demonstrates |
|---------|---------------------|
| [`bell_pair.gala`](./bell_pair.gala) | Entangled qubit pair, measurement |
| [`ghz_state.gala`](./ghz_state.gala) | Three-qubit GHZ state |
| [`quantum_teleportation.gala`](./quantum_teleportation.gala) | Quantum teleportation protocol |
| [`superdense_coding.gala`](./superdense_coding.gala) | Superdense coding (2 classical bits via 1 qubit) |
| [`qft.gala`](./qft.gala) | 2-qubit Quantum Fourier Transform |
| [`grover.gala`](./grover.gala) | Grover's search over 2 items |
| [`phase_estimation.gala`](./phase_estimation.gala) | Single-counting-qubit phase estimation |
| [`trotter.gala`](./trotter.gala) | First-order Trotter step |
| [`toffoli.gala`](./toffoli.gala) | Toffoli gate decomposition |

## Hybrid (Quantum-Classical)

| Example | What it demonstrates |
|---------|---------------------|
| [`classifier.gala`](./classifier.gala) | Variational quantum classifier |
| [`gradient.gala`](./gradient.gala) | Native `grad` operator for parameter-shift gradients |

## Related

- [docs/TUTORIAL](https://github.com/gala-pl/gala/tree/main/docs/TUTORIAL) — a five-module teaching path
- [cookbook/](https://github.com/gala-pl/gala/tree/main/cookbook) — recipe-style solutions
- [tests/conformance/](https://github.com/gala-pl/gala/tree/main/tests/conformance) — executable specification
