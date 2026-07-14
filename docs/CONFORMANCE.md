# Conformance

SPEC Chapter 23 defines conformance levels. This toolkit claims
`CompleteImplementation` via `toolkit_claim()` / `implemented_levels()` when the
coverage matrix is Gap-free and the Appendix E suite is green.

## Levels

Parser · Validator · Planner · Capability Evaluator · Orchestrator Binder ·
Registry · Complete Implementation

## Run the suite

```bash
make conformance
# or
cargo test -p dpcs --all-features --test conformance --test conformance_appendix_e
```

Appendix E areas covered in `tests/conformance_appendix_e.rs` include parser,
COM, validation, data/control flow, planning (incl. default resolve + nesting),
capabilities, binding (`dpcs_semantics.json`), diagnostics meta, compatibility
(`compare_contracts` / `compare_plans`), versioning, extensions, registries,
security/governance, and profile/claim validation.

## Profile example

```bash
dpcs conformance validate examples/conformance.profile.yaml
dpcs version --json   # includes toolkit ConformanceClaim
```

## Coverage map

Chapter-level status: [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md).  
Diagnostic identifiers: [`diagnostics.catalog.json`](diagnostics.catalog.json)
(`make diagnostics-catalog`).

Same-RI golden COM interchange (YAML ↔ JSON equality) is covered by
`tests/round_trip.rs`. A multi-implementation interoperability suite remains
Spec-optional future work.
