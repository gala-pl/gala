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
cargo test --workspace                # 275 tests, all passing at the last full run
cargo fmt --check
cargo doc --workspace --no-deps
./target/debug/gala run tests/conformance/library/grover_search.gala   # [true, true]
cd tools/gala-mcp && bun test         # 11 pass
cd sdk/typescript && bun test         # 6 pass
cd sdk/python && PYTHONPATH=src GALA_BIN=<repo>/target/debug/gala python -m unittest tests.test_agent tests.test_bridge tests.test_kernel   # 13 pass
```

The Python bridge and notebook tests need PyTorch and ipykernel. They ran in a scratch virtualenv
(torch 2.2.2), not a project dependency.

Conformance corpus: `tests/conformance/**/*.gala` (58 cases). Each case states its expected
outcome in a header: `// expect: ok`, `// expect: E0412`, `// expect-output: ...`, or
`// expect-run-error: E0603`.

## Assumptions and decisions

- **A1 Toolchain.** The pinned `stable-2026-04-14` is not installed here; builds use 1.95.0.
  The pin is left untouched.
- **A2 Commits.** The first snapshot was committed on `wp-010-frontend-to-simulator` (`c41e312`,
  `feat: finish open work packages across simulator, tooling, and library`). The changes of
  round 4 are in the working tree and not committed.
- **A3 Error codes.** `E` + four digits; the two digits after `E0` select the category
  (`E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity, `E05xx` uncomputation,
  `E06xx` backend, `E07xx` runtime). Assignments: E0401 no-cloning, E0402 implicit drop,
  E0403 aliasing, E0405 branch mismatch, E0412 use after consume (`TYPE_SYSTEM.md` §1 example),
  E0530 cannot uncompute, E0601 capability (base profile cannot branch on a measurement),
  E0602 QIR emission refusal, E0603 hardware backend unavailable, E0701 runtime, E0702 capacity.
- **A4 Simulator.** The docs name roqoqo/QuEST. The default backend is a pure-Rust exact
  state-vector simulator (deterministic under a seed, no C toolchain). A roqoqo-quest backend is
  not built. See `docs/decisions/0002`.
- **A5 QIR.** `llvm-config` is absent, so `inkwell` cannot build. QIR is emitted as textual LLVM
  IR. See `docs/decisions/0003`. Classical native code is generated as Rust source and compiled
  with `rustc` (`gala-codegen-classical`), not through LLVM.
- **A6 Conflicting docs.** `AGENTIC_BUILD_PLAN` (more specific, more recent) wins over
  `TYPE_SYSTEM` where they differ. Its `docs/0N-*.md` names map to the current `docs/*.md` names.
- **A7 External actions.** Publishing (npm, PyPI, crates.io), vendor hardware, an external RFC,
  and a taught course need people or accounts outside this machine. Nothing is published.
- **A8 Parser and lexer.** Hand-written, not logos/chumsky: the parser recovers from errors and
  the lexer tracks spans. See `docs/decisions/0001`.
- **A9 Language subset.** Implemented: `fn`, `import`, top-level `let`/`const`, `if`, `for`
  over ranges and vectors, `while`, `match` on literals, tuples, `Vec<T>`, `Qubit`, `Qubits<N>`,
  `Measured<T>`, effects, `#[attr(...)]`, compound assignment, and functions with const generic
  parameters (`fn qft<const N: Int>(q: Qubits<N>)`), instantiated per explicit size (`qft<4>(q)`).
  Not implemented: type generics, structs, enums, traits, impls, lambdas, method calls, function
  types, inferred generic sizes, and `grad` as an expression (it reports E0207). Reported as
  E0207/E0101, never silently accepted.
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
- **A16 Simulation of large circuits.** Wide circuits are split into connectivity components
  and each component is simulated exactly (`gala_sim::sample_factored`). This is not a
  tensor-network (MPS) simulator: a single entangled component is still limited to 24 qubits.

---

## M0 — Bootstrap

- [x] **WP-000 Workspace skeleton.** Builds with all crates present. `gala --version` prints.
  Test: CI-equivalent build and `cargo test --workspace`.
- [x] **WP-001 Span, source DB, interning.** `gala-span` tests: `span_arithmetic_and_display`,
  `line_col_counts_newlines`, `files_iterate_in_id_order`, `interner_empty_string_round_trips`,
  `test_interner`. Byte offsets are used throughout.
- [x] **WP-002 Diagnostics framework.** Code registry with long-form text, ariadne rendering, `diagnostic.v1` JSON,
  `gala explain` (text, Markdown, JSON). Tests: `json_matches_schema_v1_shape`, `every_registered_code_has_an_explanation`,
  and the insta snapshot `multi_label_linearity` in `gala-diagnostics/tests/render_snapshot.rs`.
- [~] **WP-003 CI + repo hygiene.** `ci.yml` calls `gala check`, which exists now. Open: the PR
  template DoD checklist and branch protection are not verified here; a failing test was not
  demonstrated to block merge.

## M1 — Frontend that type-checks

- [x] **WP-010 Lexer.** Nested block comments, Greek identifiers, complex literals, invalid
  characters reported (E0105), `0..3` lexed as a range. Non-ASCII spans fixed (byte offsets).
  Tests: lexer unit and property tests; `non_ascii_text_keeps_spans_aligned`.
- [x] **WP-011 Parser + AST.** Full subset (A9). Error recovery yields a partial AST plus E01xx.
  A block-like expression at the end of a block is its tail value (`fn f() -> Int { if … }`).
  Tests: parser unit tests (`parses_bell_pair_from_spec`, `syntax_error_recovers_and_keeps_later_items`,
  `missing_brace_reports_unexpected_eof`, `parses_match_arms`, …), snapshot and property tests.
  Open: `tests/fixtures/*.gala` are not all checked by a test.
- [~] **WP-012 HIR, desugar, resolution, modules.** Name resolution (E0202), forward calls, intrinsic binding,
  imports from bundled and on-disk package modules, and const-generic instantiation (`f<4>(…)` lowers a
  fresh copy of the body with the size substituted; one instance per distinct size). Tests: `gala-hir`
  tests; multi-file package tests `import_resolves_to_a_sibling_module` and
  `missing_name_in_a_module_is_reported_at_the_import` (`gala-driver/tests/multi_file.rs`); conformance
  `library/qft_generic_roundtrip`, `library/qft_generic_three`, `library/qft_generic_matches_concrete`.
  Incremental queries: `gala-driver/src/query.rs` runs the checker as a salsa 0.27 tracked query; the test
  `unchanged_text_is_served_from_the_cache_and_changes_rerun_the_query` shows unchanged text is not rechecked.
  Open: type generics; inferred (not explicit) sizes.
- [x] **WP-013 Type & effect inference** `[spec-sensitive]`. Lattice `pure ⊑ quantum ⊑ prob`
  with a fixpoint over the call graph. Codes E0201–E0207, E0301–E0304. Tests: `pure_calling_quantum_is_e0302`,
  `quantum_calling_prob_is_e0303`, `measure_in_quantum_is_e0304`, `effects_propagate_through_calls_in_fixpoint`,
  `register_size_literal_is_checked`, `constant_index_out_of_range_is_e0206`, and 31 linearity/type tests.
  Const-generic functions are checked per instance; their declarations are not checked as written.
  Open: type generics (A9).
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
- [~] **WP-016 GIR + lowering.** Block-structured GIR, text dump, `gir.v1` JSON. Golden snapshots `bell_pair_gir`,
  `classical_gir`. Structural verifier `gala_gir::verify`, asserted in debug builds on every compile and tested
  (`lowered_programs_satisfy_the_invariants`, `a_broken_jump_is_reported`). Open: golden QFT snapshot.
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
- [x] **WP-022 Basic optimizer.** Self-inverse cancellation, rotation merging for constant angles (looking past
  constant loads), and single-qubit fusion into Euler ZYZ form (`gala-opt`). Tests:
  `cancellation_preserves_measurement_statistics`, `adjacent_rotations_merge_and_keep_statistics`,
  `euler_decomposition_reproduces_random_unitaries`, `fused_run_keeps_the_measurement_statistics`.
- [~] **WP-023 `gala.core` + `gala.gates`.** Declarations bind to builtins. Unitarity:
  `matrices_are_unitary`. Adjoints: exercised by the uncompute path (`uncompute_returns_ancilla_to_zero`).
  Open: no property test over the gate library as a whole.
- [~] **WP-024 `gala.algorithms`.** QFT2/IQFT2 and generic `qft<N>`/`iqft<N>` (round trips at N=3 and N=4, and
  the generic N=2 forward QFT checked against the concrete `iqft2`: `qft_roundtrip`, `qft_generic_*`), Grover2
  (`grover_search`), Toffoli, full adder (4 conformance cases `library/adder_*`), Trotter step (`trotter_identity`),
  one-counting-qubit phase estimation (`phase_estimate`). Open: the adder is fixed-size; Trotter has no physics
  check beyond the identity at dt=0; no `reversible`/`uncomputes` property sweep over the library.
- [x] **WP-025 Formatter.** Idempotent, comment-preserving. Every `.gala` file in the repository, including the library,
  passes `gala fmt --check` (checked this session). Tests: `formatting_is_idempotent_on_messy_input`, `is_idempotent`,
  `keeps_comments_and_strings_intact`.
- [~] **WP-026 REPL.** `gala repl`: definitions persist, expressions are evaluated on the
  simulator. Open: quantum state does not persist across lines; no visualization.
- [~] **WP-027 Property harness.** `gala test --property --json`: `effect_honesty`, `uncomputes`, `grad_matches`,
  `unitary`, `reversible`. `unitary` and `reversible` are computed exactly from basis-state action (no sampling); tests
  `x_gate_maps_basis_zero_to_basis_one`, `dropping_a_nonzero_input_is_refused_at_release`. Open: no shrinking of
  counterexamples; the function must take one `Qubit`/`Qubits<N>` (N<=6) and return the same shape.

## M3 — Hybrid & differentiable

- [x] **WP-030 Native differentiation** `[spec-sensitive]`. Parameter-shift for rx/ry/rz whose angle
  is the parameter (A9 limits arithmetic use), interprocedural through calls. Tests: `parameter_shift_matches_finite_difference`,
  `analytic_gradient_of_single_ry` (matches −sin(x+θ)), `arithmetic_use_is_rejected` (E0207). CLI: `gala grad`.
  Open: reverse-mode autodiff for classical code; `grad` as a language operator (needs function values).
- [~] **WP-031 `gala.vqa`.** Encoder, ansatz, and classifier. `classifier_trains_to_target_accuracy`
  trains to 100% with decreasing loss. Open: the training loop is in Rust (`gala_diff::train`),
  not Gala source; no QAOA/VQE templates; no optimizer zoo.
- [~] **WP-032 Noisy simulator.** Depolarizing noise per gate (`SimConfig::depolarizing`), selected in Gala with
  `backend.noisy(p)`. Tests: `depolarizing_noise_flips_a_flipped_qubit_at_the_expected_rate`,
  `depolarizing_error_flips_back_two_thirds_of_the_error_rate` (`gala-driver/tests/backend.rs`). Open: `gala.noise`
  as a typed library (not bundled); readout error.
- [x] **WP-033 Gradient batching.** `gala_diff::gradient_batched` submits all shifted circuits in one batch
  (one submission). Test: `batched_gradient_equals_unbatched_in_one_submission` (same result as the per-shift path).
- [~] **WP-034 LSP v1.** `crates/gala-lsp`: JSON-RPC over stdio with Content-Length framing (no
  tower-lsp: it is not in the offline cache). Lifecycle, diagnostics (UTF-16 positions), hover, definition,
  completion. Tests: 23 unit + `tests/stdio.rs::binary_serves_diagnostics_and_hover_over_stdio` (passing).
  Open: stdlib diagnostics map to 0:0; imported library symbols are not indexed; compile warnings dropped.
- [~] **WP-035 VS Code extension.** The client starts `gala-lsp` over stdio with `vscode-languageclient` 9.0.1
  (setting `gala.lspPath`). Checked: JavaScript syntax, manifest JSON, and the server's `initialize` reply. Not run
  in VS Code (not available here). Open: the TextMate grammar and circuit lens were not reviewed here; publishing is `[!]`.
- [~] **WP-036 `gala doc`.** `crates/gala-doc` (`document_source`, `document_file`): Markdown from `///`
  comments above `fn`. Tests: 8 passing. Open: not wired into the CLI; stdlib publication is not done.

## M4 — Hardware & scale

- [~] **WP-040 QIR emission** `[spec-sensitive]`. Base and adaptive profiles; mid-circuit branches
  use `read_result`; release emits inverse gates; CP decomposed. Tests: `bell_pair_is_base_profile_with_static_qubits`,
  `mid_circuit_measurement_selects_adaptive_profile`, `emitted_gates_match_source_on_simulator`,
  `release_emits_inverse_of_entangling_gates`, `loops_are_rejected_with_e0602`. Open: no QIR
  parser or external QIR consumer validates the text; `llvm` feature not implemented.
- [!] **WP-041 Vendor backends via `qoqo_qir`.** Needs vendor credentials and hardware.
- [~] **WP-042 Backend selection.** `run(f, on: backend, shots: n)` with `backend.simulator`, `backend.base`,
  `backend.noisy(p)` and `backend.hardware`. A base-profile backend refuses programs that branch on a measurement
  (E0601, `types/backend_base_branch`); the hardware backend reports E0603 unavailable (`run/backend_hardware`).
  Tests: `simulator_measures_every_shot_true`, `noiseless_backend_matches_the_simulator_exactly`,
  `depolarizing_error_flips_back_two_thirds_of_the_error_rate`. Open: `run` takes a named function, not a
  function value (A9); real hardware is `[!]` (WP-041).
- [~] **WP-043 Target lowering.** SWAP routing on line and ring topologies with exact equivalence check, SWAP→CX
  decomposition, depth. Open: hardware native-gate sets; wiring into `gala build` (not done).
- [~] **WP-044 Classical codegen.** `gala build --emit rust` and `--emit native` (needs `-o`) translate GIR to Rust
  with a dynamic value runtime and compile it with `rustc -O` (`gala-codegen-classical`). Tests: `native_output_matches_the_simulator`,
  `native_loop_runs_and_matches_the_interpreter`. Open: the values are dynamic (`V`), not typed; no LLVM backend.
  The stale `llvm_codegen*.rs` files in that crate are not compiled and remain on disk.
- [~] **WP-045 Large-circuit simulation.** Connectivity-factorized exact sampler (A16): a 40-qubit circuit made of
  disjoint Bell pairs runs where a state vector cannot (`forty_qubits_of_bell_pairs_run_where_a_state_vector_cannot`).
  Open: this is not a tensor-network (MPS) simulator; an MPS needs an SVD the workspace does not provide offline.
- [x] **WP-046 Package manager + registry v1.** Local registry: publish and consume, reproducible
  lockfile. Tests (gala-pkg): `test_publish_and_consume_end_to_end`, `resolver_conflict_names_both_requirements`,
  `locked_install_fails_when_manifest_and_lockfile_disagree`, `publish_refuses_remote_registries_and_opening_them`.
  Remote publication is refused by design.
- [~] **WP-047 Hardware-aware optimizer.** Cost-aware routing (Dijkstra over per-edge costs, `route_weighted`);
  unit costs reproduce plain routing (`unit_costs_match_plain_routing_swap_count`), ring routes no further than line
  (`ring_routes_no_further_than_line`). Open: hardware native-gate sets; wiring into `gala build`.

## M5 — Ecosystem & FT readiness

- [x] **WP-050 Adaptive control flow.** Measure-and-branch runs on the simulator and as adaptive QIR. Tests:
  conformance `run/adaptive_branch`, `mid_circuit_measurement_selects_adaptive_profile`.
- [x] **WP-051 Python bridge.** `gala_agent.bridge`: `GalaKernel` computes expectations (`gala call`) and gradients
  (`gala grad`); `GalaExpectation` is a `torch.autograd.Function`. Tests: `test_expectation_matches_the_analytic_value`,
  `test_gradient_matches_the_analytic_derivative`, `test_pytorch_loop_trains_a_gala_kernel` (needs PyTorch).
- [~] **WP-052 Notebook kernel.** `gala_agent.kernel`: an ipykernel that evaluates Gala definitions, expressions,
  and `%run` cells; install with `python -m gala_agent.kernel --install`. Tests: `test_definitions_persist_and_expressions_evaluate`,
  `test_bad_cell_reports_an_error`, `test_run_magic_shows_a_histogram` (needs ipykernel). Not run in a real Jupyter UI.
- [~] **WP-053 Expanded stdlib.** Stability tiers (`docs/STDLIB_STABILITY.md`) and a changelog entry. The stdlib gained
  algorithms (WP-024); no new property-test sweep yet.
- [~] **WP-054 Formal core writeup.** `docs/FORMAL_CORE.md`: semantics and progress/preservation sketch,
  explicitly not mechanized, with a "not proven" section. Cross-links to TYPE_SYSTEM.md are listed for the lead.
- [~] **WP-055 Governance + RFC process.** `docs/decisions/` ADRs 0001–0006 (checked against the source)
  and `docs/RFC_TEMPLATE.md`. External RFC landing is `[!]`.
- [!] **WP-056 Education program.** `docs/TUTORIAL/` modules 01–05, each sample checked by the compiler.
  The M5 gate (a course that teaches Gala end to end) needs human validation.

## Agent track (WP-060..072)

- [~] **WP-060 Structured diagnostics JSON + `gala check --json`.** `gala.check.v1` envelope; each diagnostic in
  `diagnostic.v1` shape; validated fixes attached to the diagnostics they repair (E0203, E0402). Open: the 80% target
  for E04xx/E05xx machine-applicable fixes is not met (only E0402 and E0203 have recipes).
- [~] **WP-061 Suggested fixes.** `gala fix [--apply] [--json]` and `check --json`. Recipes: E0203, E0402, each
  validated by recompiling. Tests: `immutable_assignment_gets_mut_fix`, `unconsumed_qubit_gets_drop_before_scope_end`,
  `fix_is_not_offered_when_it_would_add_an_error`. Open: no recipes for E0403/E0412; fixes are checked against the entry
  file alone.
- [~] **WP-062 GIR JSON schema v1.** `gala build --emit gir=json` emits `gir.v1`; `gala run --from-gir` runs a saved file.
  Test: `gir_json_round_trips_to_an_identical_program`. Open: golden JSON snapshots; spans and types are not restored on read.
- [~] **WP-063 LSP custom methods.** `gala/typeAtPosition`, `gala/effectAtPosition`, `gala/girAtPosition`
  implemented (test: `custom_methods_report_type_effect_and_gir`). `gala/suggestFixes` returns an empty
  array: the compiler generates no fixes yet.
- [x] **WP-064 Canonical formatter.** `--check`, `--diff`, `--stdin`; idempotent; all repository `.gala` files pass
  `--check` (checked this session).
- [x] **WP-065 Property JSON.** `gala test --property --json` runs `effect_honesty`, `uncomputes`, `grad_matches`,
  `unitary`, `reversible`; output has the `property_result.v1` fields. Checked manually this session.
- [x] **WP-066 `gala explain --markdown/--json`.** Tests: `markdown_and_json_render_all_registered_codes`,
  `explain_known_code_mentions_title`.
- [~] **WP-067 Corpus generator.** `gala corpus <dir>` emits JSONL (`prompt`, `completion`, `properties`) for every
  conformance case. Open: `properties` holds the expectations, not property results; no agent-quality evaluation.
- [~] **WP-068 `gala pkg --json`.** `gala pkg add|remove|update|install|publish` print JSON (schema `gala-pkg.v1`).
  Checked: `pkg install` in a fresh project; `pkg add` against a missing registry fails cleanly. Open: a CI check of
  lockfile determinism.
- [x] **WP-069 `gala repl --json`.** Definitions persist; expressions are evaluated per line (stateless
  quantum state).
- [x] **WP-070 `gala check --json --fail-on-warnings`.** Exit 0 clean, 1 on diagnostics, JSON always emitted.
- [~] **WP-071 `gala-mcp` server.** `tools/gala-mcp`: MCP over stdio, tools, resources, prompts, schema validation.
  Wired to the CLI for `gala_fix`, `gala_gir`, `gala_build` (emit_gir), `gala_test` (properties), and `gala_lsp_query`
  (hover, completion, definition, GIR at position, served by the real `gala-lsp`). `bun test`: 11 pass. Not published.
- [~] **WP-072 Agent SDKs.** `sdk/typescript` (`bun test`: 6 pass) and `sdk/python` (13 tests pass) wrap the CLI.
  `grad(...)` runs `gala grad`; `verifyProperties` runs the property runner and reports `passed`/`failed`/`unsupported`/`error`.
  Not published.

## Blocked (needs people, accounts, or hardware outside this machine)

- [!] **WP-041** vendor backends (credentials and hardware).
- [!] **WP-055** external RFC landing. **WP-056** human validation of the course.
- [!] **Publication** to npm, PyPI, or crates.io (A7). Nothing is published; the user must approve it.
- [!] **VS Code runtime check** of the extension: VS Code is not available here.

## Log

- 2026-10-22: MCP server and SDKs, LSP, governance/tutorial/gala doc merged from agent work (verified:
  bun/python suites, 250 Rust tests, clippy, fmt, cargo doc all clean). Earlier: frontend rebuilt (parser, HIR, types/effects/linearity, uncompute, GIR, simulator,
  driver, CLI); conformance corpus; library; gala-diff; gala-qir; gala-fmt; gala-pkg merged from
  a worktree; noise; routing; REPL; property runner; byte-offset span fix.
- 2026-10-22 (later): `gala fix`, `fmt --diff/--stdin`, GIR JSON round trip with `run --from-gir`,
  `gala corpus`, adaptive conformance case. Gates: fmt, clippy, cargo test (255 passing) clean.
- Round 3: gala fix, fmt --diff/--stdin, GIR JSON round trip, corpus, pkg CLI with JSON, MCP/SDK wiring to the CLI
  (fix, gir, property runner), LSP client in the VS Code extension, unitary/reversible property runners,
  algorithms (Toffoli, full adder, Trotter step, phase estimation), rotation merging, GIR verifier, stability tiers.
  Gates: fmt, clippy -D warnings, cargo doc, 263 Rust tests, 11 MCP tests, 6 TS SDK tests, 7 Python SDK tests — all passing.
  Not committed in round 3.
- Round 4: backends (`run ... on:`, E0601/E0603), `shots`, noisy backend, salsa query layer, gate fusion, batched gradients,
  cost-aware routing, factorized sampler (not MPS), native Rust codegen, `gala grad`, Python bridge and notebook kernel,
  MCP LSP query, VS Code LSP client, const-generic functions (`qft<N>`, `iqft<N>`). Gates at the last run: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (275 passing), `cargo doc --workspace --no-deps`
  clean; `bun test` in `tools/gala-mcp` (11) and `sdk/typescript` (6); Python (13). Not committed in round 4.
