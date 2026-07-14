# Implementation Phases

## Phase 1 — Skeleton

- Create Rust crate.
- Add module structure.
- Add basic README.
- Add CLI binary.
- Add fixtures.

## Phase 2 — Canonical Object Model (complete in 0.2.0)

Implement the DPCS COM core (ROADMAP 0.2.0, SPEC Ch 1–4):

- PipelineContract
- PipelineInterface
- Metadata
- Identity model (`ObjectId`, `PipelineIdentity`, `IdentityCatalog`, …)
- Serialization-independent COM (`ExtensionValue`, `ExtensionMap`)
- COM invariant validation

Pipeline graph slots were deepened in ROADMAP 0.4.0 (`PipelineGraph`,
`PipelineStep`, `ContractReference`, `DataFlow`, `ControlFlow`, plus
`DependencyGraph` analysis). Execution-model COM shipped in ROADMAP 0.6.0:

- QualityGate
- FailureSemantics
- PipelineLineage
- ExecutionRequirements
- SchedulingIntent

## Phase 3 — Parsing (complete in 0.3.0)

- YAML parsing with Parse-stage diagnostics
- JSON parsing with Parse-stage diagnostics
- Extension preservation (root and nested)
- Parse diagnostics (`DPCS-PARSE-001`, `DPCS-PARSE-002`)
- Round-trip serialization (`to_yaml`, `to_json`) and tests

## Phase 3.5 — Pipeline Graph (complete in 0.4.0)

- PipelineGraph entry/exit points and metadata
- DependencyGraph traversal, cycle detection, dependency analysis
- Topological planning skeleton wired from DependencyGraph
- Graph diagnostics `DPCS-GRP-001`–`DPCS-GRP-008`

## Phase 4 — Diagnostics (complete)

Implement deterministic diagnostics:

- id
- severity
- stage
- category
- message
- object reference
- remediation

## Phase 5 — Validation (complete in 0.5.0–0.6.0)

Phase-based validation:

1. Document validation
2. Canonical Object Model validation
3. Structural validation
4. Graph validation
5. Reference validation (including `transformRef` and step-port refs)
6. Data Flow validation (dataset identity, wiring, reachability)
7. Control Flow validation (conflicts and duplicates)
8. Execution / Scheduling / Quality / Failure / Lineage (complete in 0.6.0)
9. Extensions / security / governance (complete in 0.9.0)

## Phase 6 — CLI

Commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs capabilities <profile> --plan <contract>
dpcs bind <contract> --profile <profile> --target <airflow|dagster|prefect|temporal|kubernetes>
dpcs version
```

## Phase 7 — Pipeline Plan (complete in 0.6.0)

`PipelinePlan` captures resolved steps, graph, contract references, dependency
edges, deterministic `stepOrder`, and preserved execution/scheduling/quality/
failure/lineage intents. Planning is gated on successful validation.

## Phase 8 — Capability Model (complete in 0.7.0)

`CapabilityProfile` declares orchestrator supply. `evaluate` / `evaluate_requirements`
match plan or execution demands without mutating the plan. CLI
`dpcs capabilities` reports match results.

## Phase 9 — Orchestrator Binding (complete in 0.8.0)

`bind` / `bind_contract` capability-gate a plan, then emit scaffold artifacts via
Airflow, Dagster, Prefect, Temporal, and Kubernetes adapters. CLI `dpcs bind`
writes artifacts and reports `BindingBundle` JSON when requested.

## Phase 10 — Complete Specification (complete in 0.9.0)

Closes SPEC Chapters 18–25:

- Diagnostics report metadata and related identifiers
- SemVer-compatible versioning validation
- Extension namespace rules (`DPCS-EXT-*`)
- Compatibility analysis (`compare_contracts` / CLI)
- Security and governance metadata
- Registry document model (ADR-0004; no network client)
- Conformance profiles/claims and `tests/conformance` suite

## Phase 11 — Ecosystem (complete in 0.10.0)

- JSON Schema / OpenAPI helpers (`schemas/`, `dpcs schema`)
- Pipeline packages (`.dpcspkg`, `dpcs package`)
- Reference registry HTTP API / client / server (ADR-0005)
- Python bindings (PyO3 / maturin → PyPI `dpcs`)
- WASM bindings (wasm-bindgen → npm `@eddiethedean/dpcs`, Wasmer `eddiethedean/dpcs`)

Distribution details: [`BINDINGS.md`](BINDINGS.md).

## Phase 12 — Developer experience (complete in 0.11.0)

- Report module: Markdown / HTML / Mermaid / DOT exports
- Rich CLI `--format` / `--out` (with `--json` alias)
- Interactive TUI inspector (`tui` feature; `dpcs tui` / `inspect --tui`)

## Phase 13 — Performance (complete in 0.12.0)

- Shared `AnalysisContext` + large-graph endpoint/graph optimizations
- Parallel validation (`parallel` feature) and `validate_sequential`
- Incremental `ValidationCache` / `validate_cached`
- Practical allocation cuts (wire serialize without full COM clone)
- Criterion benches + `dpcs::synth` generators

## Phase 14 — Reference implementation (complete in 0.13.0)

- SPEC coverage matrix ([`SPEC_COVERAGE.md`](SPEC_COVERAGE.md)) + diagnostic catalog
- Contract reference resolution (`resolve` module) and nested pipeline planning
- Structured bind semantics (`dpcs_semantics.json` on all targets)
- Appendix E conformance suite (`make conformance`)
- Public API stability documentation; remove deprecated `OrchestratorCapabilities`
- CompleteImplementation claim restored; comprehensive guides
