# Learn Gala

A five-module path through Gala, built around the compiler's teaching diagnostics. Each module is
short, and every program in it is one the compiler accepts, except where a module shows a
deliberate error and its real diagnostic code.

> **Validation status.** The programs in these modules are checked by the compiler, not by
> people. The *human course validation* part of WP-056 (a cohort working through the path end to
> end, and the M5 gate in [AGENTIC_BUILD_PLAN.md](../AGENTIC_BUILD_PLAN.md)) is **not done**. It is
> blocked on people (learners and instructors) and cannot be completed by an agent. Treat this
> path as a draft until that validation has run.

## Modules

1. [01-hello-classical.md](./01-hello-classical.md): functions, `print`, and the `pure` effect.
2. [02-bell-pair.md](./02-bell-pair.md): qubits, gates, measurement, and entanglement.
3. [03-no-cloning-errors.md](./03-no-cloning-errors.md): reading the linearity diagnostics
   E0401, E0402, E0403, E0405, and E0412.
4. [04-effects.md](./04-effects.md): the `pure`, `quantum`, and `prob` boundary.
5. [05-hybrid-classifier.md](./05-hybrid-classifier.md): a trainable classifier, using
   `gala.vqa` and the `gala-diff` training API.

## Checking the samples yourself

From the repository root:

```
cargo build --bin gala
./target/debug/gala check <file.gala>      # type, effect, and linearity check, no run
./target/debug/gala run <file.gala>        # run on the simulator
./target/debug/gala explain E0412          # long-form explanation of a code
```

`gala check` exits with status 0 on success and 1 on error. `--json` prints diagnostics in the
`diagnostic.v1` schema.

The conformance suite under `tests/conformance/` is the executable definition of this behavior.
The samples here are drawn from it or match it.
