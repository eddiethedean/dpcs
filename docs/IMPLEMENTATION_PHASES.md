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
9. Extension stubs (namespace rules in 0.9.0)

## Phase 6 — CLI

Commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs version
```

## Phase 7 — Pipeline Plan (complete in 0.6.0)

`PipelinePlan` captures resolved steps, graph, contract references, dependency
edges, deterministic `stepOrder`, and preserved execution/scheduling/quality/
failure/lineage intents. Planning is gated on successful validation.

Do not implement orchestrator binding yet (ROADMAP 0.8.0).
