# gala-agent (Python)

Agent-facing wrapper around the `gala` CLI, using `subprocess` and `json`. Source only:
not published to PyPI.

```python
from gala_agent import GalaAgent

gala = GalaAgent()  # $GALA_BIN, else <repo>/target/debug/gala

check = gala.check(source)                 # CheckResult(ok, errors, warnings, diagnostics, schema)
run = gala.run(source, seed=1, shots=1024) # RunResult(ok, output, result, shots, histogram, error)
grad = gala.grad("f", "theta")             # GradResult(supported=False, reason=...)
props = gala.verify_properties(source, ["unitary"])  # PropertyVerification(status="unsupported", ...)
```

## Agent usage patterns

- **Check, then fix.** Call `check` after every edit and return `diagnostics` (code, message,
  `suggested_fixes`) to the model. Run only code that passes `check`.
- **Trust the status fields.** `grad` and `verify_properties` report `unsupported` today. An
  unsupported status is not a pass.
- **Inspect `run` results.** `run` returns failures in `ok` and `error` instead of raising.
  `output` holds `print()` lines; `histogram` holds measured outcomes.

## Tests

```sh
cd sdk/python && python3 -m unittest discover -s tests -v
```

Requires the binary at `target/debug/gala` (`cargo build --bin gala`) or `GALA_BIN`.
