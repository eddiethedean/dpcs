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

The following COM slots are present as thin placeholders in 0.2.0 and are
deepened in ROADMAP 0.4.0:

- PipelineGraph
- PipelineStep
- ContractReference
- DataFlow
- ControlFlow
- QualityGate
- FailureSemantics
- PipelineLineage
- ExecutionRequirements
- SchedulingIntent

## Phase 3 — Parsing

- YAML parsing
- JSON parsing
- extension preservation
- parse diagnostics

## Phase 4 — Diagnostics

Implement deterministic diagnostics:

- id
- severity
- stage
- category
- message
- object reference
- remediation

## Phase 5 — Validation

Implement phase-based validation:

1. Document validation
2. Canonical Object Model validation
3. Structural validation
4. Graph validation
5. Reference validation
6. Data Flow validation
7. Control Flow validation
8. Extension validation

## Phase 6 — CLI

Commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs version
```

## Phase 7 — Pipeline Plan Skeleton

Add `PipelinePlan` types and a stub planner.

Do not implement orchestrator binding yet.
