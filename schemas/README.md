# Gala Schemas

JSON Schema definitions for the Gala compiler's intermediate representations and diagnostics.

## Directory layout

```
schemas/
├── README.md
└── v1/
    ├── diagnostic.json
    ├── gir.json
    └── property_result.json
```

All schemas live under `v1/` and use JSON Schema draft-07.

## Schemas

### `v1/gir.json` — Gala IR (GIR)

The compiler's intermediate representation emitted after the uncomputation pass. It describes functions, their parameter types and linearity, return types, effect annotations, and the IR body. Types and effects are also exposed as top-level maps so downstream backends can resolve named types and effect signatures without re-parsing source.

- **`version`** — schema version, fixed to `1`.
- **`functions`** — array of function definitions, each with:
  - `id`, `name`, `params` (with `linearity`: `linear` or `unrestricted`)
  - `return_type`, `effect` (`pure`, `quantum`, `prob`)
  - `body` (GIR expression node), `spans` (source location)
  - `properties` (optional flags: `is_unitary`, `is_reversible`, `has_adjoint`, `has_controlled`)
- **`types`** — map of type name to type descriptor (`kind`, `name`, `params`, `linearity`). Kinds include `primitive`, `qubit`, `qubits`, `measured`, `function`, `array`, `tuple`, `struct`, `named`.
- **`effects`** — map of effect name to effect kind (`pure`, `quantum`, `prob`).
- **`metadata`** — optional `source_hash`, `compiler_version`, `timestamp`.

### `v1/diagnostic.json` — Diagnostic

A single compiler diagnostic (error, warning, note, or help). Mirrors the structure used by rustc and similar toolchains so that IDEs and language servers can render diagnostics consistently.

- **`code`** — error code matching `E\d{4}`.
- **`severity`** — `error`, `warning`, `note`, or `help`.
- **`message`** — human-readable description.
- **`primary_span`** — the source span the diagnostic points at (`file_id`, `start`, `end`, optional `file_path`).
- **`labels`** — additional annotated spans with their own messages.
- **`notes`** — free-form explanatory notes.
- **`suggested_fixes`** — machine-readable edits (`description` + list of `file_id`/`start`/`end`/`new_text` changes).
- **`explain_url`** — optional URI pointing to documentation explaining the error code.

### `v1/property_result.json` — Property Test Result

The outcome of a property-based check run on a compiled function. Used by the verification harness to report whether a function satisfies a quantum-law property.

- **`property`** — one of `unitary`, `reversible`, `uncomputes`, `grad_matches`, `effect_honesty`.
- **`passed`** — whether the property held.
- **`details`** — optional fields: `function_name`, `num_trials`, `tolerance`, `max_error`, `counterexample`, `matrix_norm_diff`, `fidelity`.
- **`duration_ms`** — how long the check took.

## Usage

These schemas are referenced by the compiler and tooling via their `$id` URIs:

- `https://gala-lang.org/schemas/gir.v1.json`
- `https://gala-lang.org/schemas/diagnostic.v1.json`
- `https://gala-lang.org/schemas/property_result.v1.json`

They can be used directly with any JSON Schema validator, or registered with an editor/language server to enable validation of compiler output.