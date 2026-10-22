# Standard library stability tiers

Each bundled module carries a stability tier (see `docs/STANDARD_LIBRARY.md`, "Stability tiers").

| Module | Tier | Why |
|--------|------|-----|
| `gala.core` | stable | Declarations that bind directly to compiler builtins: `qubit`, `qubits`, `measure`, `drop`, `print`. |
| `gala.gates` | stable | Declarations that bind directly to unitary gate builtins. Adjoints come from the compiler. |
| `gala.algorithms` | unstable | Written in Gala with fixed register sizes (`Qubits<2>`, `Qubits<4>`) because generic sizes are not supported yet. Signatures may change when they are. |
| `gala.vqa` | unstable | One-qubit encoder and ansatz; multi-qubit templates are not implemented. |

Not bundled: `gala.noise`, `gala.hardware`, `gala.classical`. Their files under `library/std/` predate the
compiler and are not implemented; they are not importable until they are.

Breaking changes to a stable module need an RFC (`docs/RFC_TEMPLATE.md`) and a changelog entry.
