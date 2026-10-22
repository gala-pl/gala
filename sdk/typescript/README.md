# @gala-lang/agent (TypeScript)

Agent-facing wrapper around the `gala` CLI. Source only: `private: true`, not published.

```ts
import { GalaAgent } from './src/index.ts';

const gala = new GalaAgent(); // $GALA_BIN, else <repo>/target/debug/gala

const check = await gala.check(source);       // { ok, errors, warnings, diagnostics[], schema }
const run = await gala.run(source, { seed: 1, shots: 1024 });
//   { ok, output: string[], result, shots, histogram, stderr, error? }
const grad = await gala.grad('f', 'theta');   // { supported: false, reason } (no gradient command yet)
const props = await gala.verifyProperties(source, ['unitary']);
//   { status: 'unsupported', reason, requested, diagnostics }
```

## Agent usage patterns

- **Check, then fix.** Call `check` after every edit. Feed `diagnostics[].code`, `message`,
  and `suggested_fixes` back to the model. Do not run `run` on code that fails `check`.
- **Trust the status fields.** `grad` and `verifyProperties` report `unsupported` today. Do not
  treat a missing number or an `unsupported` status as a pass.
- **Read `run` results structurally.** `run` never throws on a Gala-level failure; check `ok`
  and `error`. Use `output` for `print()` lines and `histogram` for measured entry points.
- **Seed for reproducibility.** Pass `seed` when comparing runs in an eval loop.

## Tests

```sh
cd sdk/typescript && bun test
```

Requires the binary at `target/debug/gala` (`cargo build --bin gala`) or `GALA_BIN`.
