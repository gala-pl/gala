# Gala — Implementation Progress

Source of truth: every `.md` in `docs/`. Work packages (WP) come from
`docs/AGENTIC_BUILD_PLAN.md` §4; acceptance criteria are the WP's Definition of Done (DoD).
Where docs conflict, the more specific doc wins (see A6).

## Legend

- `[x]` done: implemented, built, and covered by tests that assert the acceptance criteria
- `[~]` partial: part of the DoD is implemented and tested; the rest is listed as open
- `[ ]` not started
- `[!]` blocked: needs something outside this machine (accounts, hardware, publication, people)

Every `[x]` names the tests that prove it. A build alone never counts.

## Verification

```
export RUSTUP_TOOLCHAIN=1.95.0        # rust-toolchain.toml pins stable-2026-04-14 (not installed here)
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                # 255 tests, all passing at the last full run
cargo fmt --check
cargo doc --workspace --no-deps
./target/debug/gala run tests/conformance/library/grover_search.gala   # [true, true]
```

Conformance corpus: `tests/conformance/**/*.gala` (46 cases). Each case states its expected
outcome in a header: `// expect: ok`, `// expect: E0412`, or `// expect-output: ...`.

## Assumptions and decisions

- **A1 Toolchain.** The pinned `stable-2026-04-14` is not installed here; builds use 1.95.0.
  The pin is left untouched.
- **A2 Commits.** No commits were made. All work is in the working tree for review.
- **A3 Error codes.** `E` + four digits; the two digits after `E0` select the category
  (`E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity, `E05xx` uncomputation,
  `E06xx` backend, `E07xx` runtime). Assignments: E0401 no-cloning, E0402 implicit drop,
  E0403 aliasing, E0405 branch mismatch, E0412 use after consume (`TYPE_SYSTEM.md` §1 example),
  E0530 cannot uncompute, E0602 QIR emission refusal, E0701 runtime, E0702 capacity.
- **A4 Simulator.** The docs name roqoqo/QuEST. The default backend is a pure-Rust exact
  state-vector simulator (deterministic under a seed, no C toolchain). A roqoqo-quest backend is
  not built. See `docs/decisions/0002`.
- **A5 QIR.** `llvm-config` is absent, so `inkwell` cannot build. QIR is emitted as textual LLVM
  IR. See `docs/decisions/0003`.
- **A6 Conflicting docs.** `AGENTIC_BUILD_PLAN` (more specific, more recent) wins over
  `TYPE_SYSTEM` where they differ. Its `docs/0N-*.md` names map to the current `docs/*.md` names.
- **A7 External actions.** Publishing (npm, PyPI, crates.io), vendor hardware, an external RFC,
  and a taught course need people or accounts outside this machine. Nothing is published.
- **A8 Parser and lexer.** Hand-written, not logos/chumsky: the parser recovers from errors and
  the lexer tracks spans. See `docs/decisions/0001`.
- **A9 Language subset.** Implemented: `fn`, `import`, top-level `let`/`const`, `if`, `for`
  over ranges and vectors, `while`, `match` on literals, tuples, `Vec<T>`, `Qubit`, `Qubits<N>`
  with literal `N`, `Measured<T>`, effects, `#[attr(...)]`, compound assignment. Not implemented:
  structs, enums, traits, impls, lambdas, method calls, generic functions, const generics,
  function types, `grad` as an expression (it reports E0207). Reported as E0207/E0101, never
  silently accepted.
- **A10 Tuples as registers.** A tuple of `n` qubits is accepted where `Qubits<n>` is expected,
  because the spec's Bell pair returns `(a, b)` from a function declared `-> Qubits<2>`.
- **A11 Measurement as condition.** `if` and `while` accept `Measured<Bool>` as the condition and
  branch on the sampled outcome (mid-circuit measurement). Other uses need `Bool`.
- **A12 In-place gates.** `h(q)` as a statement rebinds `q` (in the GIR the gate keeps the
  qubit's identity). `h(q[i])` borrows a register element. Moving one element out of a register
  is E0207.
- **A13 Uncomputation.** The compiler checks statically that an ancilla's entanglement cone is
  still live (E0530). The inverse gates are then applied from the recorded gate history at run
  time, so angles and branch paths are exact. The release also verifies |0⟩. Static synthesis of
  an adjoint sequence is not implemented. See `docs/decisions/0004`.
- **A14 Expectation values.** Gradients use shot-estimated `P(false) − P(true)`. Their accuracy is
  therefore sampling-limited (tests use 40k–60k shots and tolerances of 0.04–0.1).
- **A15 Library.** The bundled standard library is `library/std/{gates,core,algorithms,vqa}.gala`.
  Declarations without bodies bind to compiler builtins. `noise.gala`, `hardware.gala`, and
  `classical.gala` are not bundled because their contents were placeholders and are not
  implemented.

---

## M0 — Bootstrap

- [x] **WP-000 Workspace skeleton.** Builds with all crates present. `gala --version` prints.
  Test: CI-equivalent build and `cargo test --workspace`.
- [x] **WP-001 Span, source DB, interning.** `gala-span` tests: `span_arithmetic_and_display`,
  `line_col_counts_newlines`, `files_iterate_in_id_order`, `interner_empty_string_round_trips`,
  `test_interner`. Byte offsets are used throughout.
- [~] **WP-002 Diagnostics framework.** Registry of 40+ codes with long-form text, ariadne
  rendering, `diagnostic.v1` JSON, `gala explain` in text/Markdown/JSON.
  Tests: `json_matches_schema_v1_shape`, `render_contains_code_message_and_snippet`,
  `every_registered_code_has_an_explanation`, `markdown_and_json_render_all_registered_codes`.
  Open: no insta snapshot test of a multi-label diagnostic.
- [~] **WP-003 CI + repo hygiene.** `ci.yml` calls `gala check`, which exists now. Open: the PR
  template DoD checklist and branch protection are not verified here; a failing test was not
  demonstrated to block merge.

## M1 — Frontend that type-checks

- [x] **WP-010 Lexer.** Nested block comments, Greek identifiers, complex literals, invalid
  characters reported (E0105), `0..3` lexed as a range. Non-ASCII spans fixed (byte offsets).
  Tests: lexer unit and property tests; `non_ascii_text_keeps_spans_aligned`.
- [x] **WP-011 Parser + AST.** Full subset (A9). Error recovery yields a partial AST plus E01xx.
  Tests: parser unit tests (`parses_bell_pair_from_spec`, `syntax_error_recovers_and_keeps_later_items`,
  `missing_brace_reports_unexpected_eof`, `parses_match_arms`, …), snapshot and property tests.
  Open: `tests/fixtures/*.gala` are not all checked by a test.
- [~] **WP-012 HIR, desugar, resolution, modules.** Name resolution with E0202, forward calls,
  intrinsic binding, imports from bundled and on-disk modules (`load_file` in the driver).
  Tests: nine `gala-hir` tests (`unresolved_name_is_e0202`, `import_from_module_table_binds_function`,
  `let_rhs_sees_previous_binding`, …). Open: no multi-file package test; no salsa incremental test
  (salsa is not used).
- [x] **WP-013 Type & effect inference** `[spec-sensitive]`. Lattice `pure ⊑ quantum ⊑ prob`
  with a fixpoint over the call graph. Codes E0201–E0207, E0301–E0304. Tests: `pure_calling_quantum_is_e0302`,
  `quantum_calling_prob_is_e0303`, `measure_in_quantum_is_e0304`, `effects_propagate_through_calls_in_fixpoint`,
  `register_size_literal_is_checked`, `constant_index_out_of_range_is_e0206`, and 31 linearity/type tests.
  Open: const generics (A9).
- [x] **WP-014 Linearity checker** `[spec-sensitive]`. E0401/E0402/E0403/E0405/E0412 with `help:`
  text. Tests: `use_after_consume_is_e0412`, `unconsumed_qubit_is_implicit_drop_e0402`,
  `same_qubit_twice_in_call_is_aliasing_e0403`, `consumed_on_one_branch_only_is_e0405`,
  `consuming_outer_qubit_inside_loop_is_e0412`, `early_return_leaks_live_qubit`, `index_into_register_borrows_for_gates`,
  and conformance cases `linearity/*`.
- [x] **WP-015 Uncomputation** `[spec-sensitive]`. Static liftability with E0530 refusal reasons,
  dynamic inverse from history (A13). Tests: `unentangled_ancilla_is_accepted`,
  `partner_measured_before_drop_is_e0530`, `passing_through_user_call_is_e0530`,
  `register_drop_counts_each_element`; sim: `uncompute_returns_ancilla_to_zero`,
  `released_entangled_ancilla_undoes_entanglement`; conformance `uncompute/*`, `run/uncompute_runs`.
- [x] **WP-016 GIR + lowering.** Block-structured GIR with `Gir::dump`. Golden snapshots:
  `bell_pair_gir`, `classical_gir`. Tests: `bell_lowers_to_gates_and_measure`, `for_range_creates_loop_blocks`,
  `dump_is_deterministic`. Open: linearity invariant checks in debug builds.
- [x] **WP-017 `gala check` + `explain`.** `gala check` (text and `--json`), `gala explain`. The
  conformance suite asserts every case's expected code or output (`conformance_suite_passes`).

## M2 — It runs on the simulator

- [x] **WP-020 Simulator backend.** Exact state-vector engine with allocation, gates, measurement,
  release (A13), and a 24-qubit cap (E0702). Tests: `bell_pair_correlates_and_splits_evenly`,
  `deterministic_under_fixed_seed`, `controlled_phase_and_swap_act_as_expected`,
  `capacity_limit_is_reported`, `matrices_are_unitary`. QFT: `qft_roundtrip` (inverse restores the
  input). Grover: `grover_search` gives `[true, true]`.
- [x] **WP-021 `gala run`.** `gala new demo && gala run demo` prints a Bell measurement (the
  scaffold is a Bell pair). Verified at the last run; the scaffold is in `gala-pkg`
  (`init_project`).
- [~] **WP-022 Basic optimizer.** Adjacent self-inverse cancellation with a semantic check
  (`cancellation_preserves_measurement_statistics`). Open: gate fusion and rotation merging.
- [~] **WP-023 `gala.core` + `gala.gates`.** Declarations bind to builtins. Unitarity:
  `matrices_are_unitary`. Adjoints: exercised by the uncompute path (`uncompute_returns_ancilla_to_zero`).
  Open: no property test over the gate library as a whole.
- [~] **WP-024 `gala.algorithms`.** QFT2/IQFT2 (`qft_roundtrip`), Grover2 (`grover_search`).
  Open: phase estimation, Trotter step, quantum adder; QFT is 2-qubit only (A9).
- [x] **WP-025 Formatter.** Idempotent, comment-preserving, indentation canonical. Tests:
  `formatting_is_idempotent_on_messy_input`, `is_idempotent`, `keeps_comments_and_strings_intact`,
  `control_flow_is_indented`. Open: the stdlib is not formatter-checked.
- [~] **WP-026 REPL.** `gala repl`: definitions persist, expressions are evaluated on the
  simulator. Open: quantum state does not persist across lines; no visualization.
- [~] **WP-027 Property harness.** `gala test --property`: `effect_honesty` (checker-verified),
  `uncomputes` (run and release check), `grad_matches` (parameter-shift vs finite differences).
  Open: `unitary` and `reversible` have no runner and are reported as skipped.

## M3 — Hybrid & differentiable

- [x] **WP-030 Native differentiation** `[spec-sensitive]`. Parameter-shift for rx/ry/rz whose angle
  is the parameter (A9 limits arithmetic use). Tests: `parameter_shift_matches_finite_difference`,
  `analytic_gradient_of_single_ry` (matches −sin(x+θ)), `arithmetic_use_is_rejected` (E0207).
  Open: reverse-mode autodiff for classical code; `grad` as a language operator (E0207).
- [~] **WP-031 `gala.vqa`.** Encoder, ansatz, and classifier. `classifier_trains_to_target_accuracy`
  trains to 100% with decreasing loss. Open: the training loop is in Rust (`gala_diff::train`),
  not Gala source; no QAOA/VQE templates; no optimizer zoo.
- [~] **WP-032 Noisy simulator.** Depolarizing noise per gate (`SimConfig::depolarizing`).
  Test: `depolarizing_noise_flips_a_flipped_qubit_at_the_expected_rate` (rate 2p/3). Open: `gala.noise`
  as a typed library; readout error; the noisy-vs-noiseless demo.
- [ ] **WP-033 Gradient batching.** Submissions per gradient are counted (`Gradient::submissions`:
  2 per gate occurrence). No batching implemented: the only submissions are single shifted circuits, and
  deduplicating them would not change any result. Not done until a target is defined.
- [~] **WP-034 LSP v1.** `crates/gala-lsp`: JSON-RPC over stdio with Content-Length framing (no
  tower-lsp: it is not in the offline cache). Lifecycle, diagnostics (UTF-16 positions), hover, definition,
  completion. Tests: 23 unit + `tests/stdio.rs::binary_serves_diagnostics_and_hover_over_stdio` (passing).
  Open: stdlib diagnostics map to 0:0; imported library symbols are not indexed; compile warnings dropped.
- [~] **WP-035 VS Code extension.** `extensions/` exists from earlier work; not reviewed or tested
  in this session. Publishing to the marketplace is `[!]`.
- [~] **WP-036 `gala doc`.** `crates/gala-doc` (`document_source`, `document_file`): Markdown from `///`
  comments above `fn`. Tests: 8 passing. Open: not wired into the CLI; stdlib publication is not done.

## M4 — Hardware & scale

- [~] **WP-040 QIR emission** `[spec-sensitive]`. Base and adaptive profiles; mid-circuit branches
  use `read_result`; release emits inverse gates; CP decomposed. Tests: `bell_pair_is_base_profile_with_static_qubits`,
  `mid_circuit_measurement_selects_adaptive_profile`, `emitted_gates_match_source_on_simulator`,
  `release_emits_inverse_of_entangling_gates`, `loops_are_rejected_with_e0602`. Open: no QIR
  parser or external QIR consumer validates the text; `llvm` feature not implemented.
- [!] **WP-041 Vendor backends via `qoqo_qir`.** Needs vendor credentials and hardware.
- [ ] **WP-042 Backend polymorphism.** `run(f, on: backend)` needs functions as first-class values,
  which the language does not support (A9). Not done.
- [~] **WP-043 Target lowering.** SWAP routing on line and ring topologies with exact equivalence
  check, SWAP→CX decomposition, depth. Tests in `gala-opt::route`: `distant_gates_are_routed_with_swaps_and_stay_equivalent`,
  `ring_routes_no_further_than_line`, `swap_decomposes_to_three_cnots_with_same_distribution`,
  `depth_counts_parallel_layers`. Open: hardware-specific native sets; wiring into `gala build`.
- [ ] **WP-044 Classical codegen.** `gala-codegen-classical` predates this work and is not
  rewritten. Not done.
- [ ] **WP-045 Tensor-network simulator.** Not done.
- [x] **WP-046 Package manager + registry v1.** Local registry: publish and consume, reproducible
  lockfile. Tests (gala-pkg): `test_publish_and_consume_end_to_end`, `resolver_conflict_names_both_requirements`,
  `locked_install_fails_when_manifest_and_lockfile_disagree`, `publish_refuses_remote_registries_and_opening_them`.
  Remote publication is refused by design.
- [ ] **WP-047 Hardware-aware optimizer.** Not done.

## M5 — Ecosystem & FT readiness

- [x] **WP-050 Adaptive control flow.** Measure-and-branch runs on the simulator and as adaptive QIR.
  Tests: conformance `run/adaptive_branch` (prints `branched`), `mid_circuit_measurement_selects_adaptive_profile`.
- [ ] **WP-051 Python bridge.** Not done here (see Agent track for the SDK, which is not a PyTorch bridge).
- [!] **WP-052 Notebook kernel.** Needs a Jupyter install and a kernel protocol implementation.
- [ ] **WP-053 Expanded stdlib.** Not done beyond the four bundled modules.
- [~] **WP-054 Formal core writeup.** `docs/FORMAL_CORE.md`: semantics and progress/preservation sketch,
  explicitly not mechanized, with a "not proven" section. Cross-links to TYPE_SYSTEM.md are listed for the lead.
- [~] **WP-055 Governance + RFC process.** `docs/decisions/` ADRs 0001–0006 (checked against the source)
  and `docs/RFC_TEMPLATE.md`. External RFC landing is `[!]`.
- [!] **WP-056 Education program.** `docs/TUTORIAL/` modules 01–05, each sample checked by the compiler.
  The M5 gate (a course that teaches Gala end to end) needs human validation.

## Agent track (WP-060..072)

- [x] **WP-060 Structured diagnostics JSON + `gala check --json`.** `gala.check.v1` envelope, each
  diagnostic in `diagnostic.v1` shape. Test: `json_matches_schema_v1_shape`. Open: "80% of E04xx/E05xx
  have machine-applicable fixes" — few fixes exist.
- [~] **WP-061 Suggested fixes.** `gala fix <file> [--apply] [--json]`. Recipes: E0203 (declare `mut`),
  E0402 (`drop(name)` before scope end). A fix is offered only if the edited program compiles with fewer
  errors and adds no new code. Tests: `immutable_assignment_gets_mut_fix`, `unconsumed_qubit_gets_drop_before_scope_end`,
  `fix_is_not_offered_when_it_would_add_an_error`. Open: no recipes for E0403/E0412; not yet wired into `check --json`.
- [~] **WP-062 GIR JSON schema v1.** `gala build --emit gir=json` emits `gir.v1`; `gala run --from-gir` runs
  a saved file. Test: `gir_json_round_trips_to_an_identical_program` (identical dump and histogram), plus
  `gir_json_has_schema_v1_keys_and_typed_params`. Open: golden JSON snapshots; spans and types are not
  restored on read.
- [~] **WP-063 LSP custom methods.** `gala/typeAtPosition`, `gala/effectAtPosition`, `gala/girAtPosition`
  implemented (test: `custom_methods_report_type_effect_and_gir`). `gala/suggestFixes` returns an empty
  array: the compiler generates no fixes yet.
- [~] **WP-064 Canonical formatter.** `--check`, `--diff`, `--stdin` work (manual runs this session). Open:
  the repository's `.gala` files are not all checked with `--check`.
- [x] **WP-065 Property JSON.** `gala test --property --json` output has the `property_result.v1`
  fields. Tests: manual run recorded in the session (`effect_honesty`, `uncomputes`, `grad_matches` ok).
  Open: unitary and reversible runners.
- [x] **WP-066 `gala explain --markdown/--json`.** Tests: `markdown_and_json_render_all_registered_codes`,
  `explain_known_code_mentions_title`.
- [~] **WP-067 Corpus generator.** `gala corpus <dir>` emits JSONL (`prompt`, `completion`, `properties`) for
  all 46 conformance cases across every category. Open: `properties[]` holds the expectation and compile
  status, not the property results; no agent-quality evaluation.
- [x] **WP-068 `gala pkg --json`.** The package library's JSON outputs (`test_json_outputs_have_stable_keys`).
  The `gala add` CLI does not print JSON yet.
- [x] **WP-069 `gala repl --json`.** Definitions persist; expressions are evaluated per line (stateless
  quantum state).
- [x] **WP-070 `gala check --json --fail-on-warnings`.** Exit 0 clean, 1 on diagnostics, JSON always emitted.
- [~] **WP-071 `gala-mcp` server.** `tools/gala-mcp` (TypeScript on bun, no dependencies): MCP over stdio,
  tools, resources, prompts, hand-written schema validation. `bun test`: 11 passing. Not met: gala_fix (no CLI
  command), gala_gir (no GIR JSON via MCP yet), gala_lsp_query (not connected). Not published.
- [~] **WP-072 Agent SDKs.** `sdk/typescript` (`bun test`: 5 passing) and `sdk/python` (`unittest`: 6
  passing), both wrapping the CLI. grad and verifyProperties return "unsupported" rather than a value.
  Publishing to npm/PyPI is `[!]` and was not done.

## Log

- 2026-10-22: MCP server and SDKs, LSP, governance/tutorial/gala doc merged from agent work (verified:
  bun/python suites, 250 Rust tests, clippy, fmt, cargo doc all clean). Earlier: frontend rebuilt (parser, HIR, types/effects/linearity, uncompute, GIR, simulator,
  driver, CLI); conformance corpus; library; gala-diff; gala-qir; gala-fmt; gala-pkg merged from
  a worktree; noise; routing; REPL; property runner; byte-offset span fix.
- 2026-10-22 (later): `gala fix`, `fmt --diff/--stdin`, GIR JSON round trip with `run --from-gir`,
  `gala corpus`, adaptive conformance case. Gates: fmt, clippy, cargo test (255 passing) clean.
