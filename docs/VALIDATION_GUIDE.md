# Validation Guide

Validation is deterministic and phase-based. It returns a `ValidationReport`
and does not panic on invalid contracts. Phases always complete and findings are
accumulated, then sorted deterministically.

Deep reference resolution (SPEC Ch 7) is separate from the phase list and runs
via `validate_resolved` / `plan` / CLI document roots — see
[`PLANNING.md`](PLANNING.md). Full ID inventory:
[`diagnostics.catalog.json`](diagnostics.catalog.json).

## Performance (0.12.0)

- `validate` builds an [`AnalysisContext`] once (step ids, endpoints, dependency
  graph) and reuses it across phases.
- With the `parallel` feature (included in `full` / `dpcs-cli`), independent
  phases run concurrently via rayon; diagnostics are then merged and sorted.
  `validate_sequential` always runs phases in order (same final report).
- `validate_cached` + `ValidationCache` fingerprint contract sections and reuse
  clean phase diagnostics across incremental edits.
- Large-graph hot paths use shared indexes (`data_flow_step_dependency_with_indexes`,
  multi-source unreachable BFS). Synthetic helpers live in `dpcs::synth`.

## Phases (0.9.0)

1. Document — unsupported `dpcsVersion` warnings (`DPCS-DOC-002`)
2. Canonical Object Model — identity, uniqueness, interface completeness, reserved extension keys, SemVer version syntax (`DPCS-VER-*`)
3. Structural — empty step `type` (`DPCS-STR-003`), empty step port ids (`DPCS-STR-001`)
4. Graph — edges, cycles, duplicate edges, unreachable steps, entry/exit integrity
5. References — declared-id / path-like contract / transform / port references
   (deep load of nested DPCS uses `dpcs::resolve`; `DPCS-REF-007` / `DPCS-REF-008`)
6. Data Flow — endpoints, dataset identity, wiring, unreachable datasets
7. Control Flow — step endpoints, conflicting deps, duplicate control edges
8. Execution — capability/dependency/resource completeness (`DPCS-EXE-*`)
9. Scheduling — modes, events, timing consistency (`DPCS-SCH-*`)
10. Quality — criteria, outcomes, placement, contract refs (`DPCS-QG-*`)
11. Failure — scope, triggers, responses, retry (`DPCS-FS-*`)
12. Lineage — dataset/step/contract provenance refs (`DPCS-LIN-*`)
13. Security — secret refs / integrity refs (`DPCS-SEC-*`) when present
14. Governance — owner/authority/publication (`DPCS-GOV-*`) when present
15. Extensions — namespace syntax + informational preserve (`DPCS-EXT-*`); reserved collisions remain COM-012

Quality/failure **identity** (empty/duplicate ids) remains COM-owned
(`DPCS-COM-004` / `DPCS-COM-005`).

Capability matching is separate from contract validation: after a successful
`plan()`, call `evaluate` / `evaluate_requirements` against a `CapabilityProfile`
(`DPCS-CAP-*` at `CapabilityEvaluation`). Profile-only consistency uses
`validate_profile`. Matching demand is `requiredCapabilities` plus
`externalDependencies[].capability` (not environment software packages or isolation).

## Important validations today

- required root identity (`dpcsVersion`, `id`, `version`)
- unique identifiers within each addressable object kind
- unique interface port ids across inputs and outputs
- complete interface ports (`name`, `contractRef`, `purpose`) when ports are present
- valid graph edges and no prohibited cycles
- graph entry and exit points reference declared steps (`DPCS-GRP-007`, `DPCS-GRP-008`)
- no duplicate graph edges for identical `(from, to, kind)` (`DPCS-GRP-005`)
- no unreachable steps from declared entry points or indegree-zero roots (`DPCS-GRP-006`)
- resolvable `contractRef` / `transformRef` (including data-flow and step-port refs)
- every `dataFlow` entry declares a non-empty `dataset` (`DPCS-DF-004`)
- declared step inputs and interface outputs have incoming data flows (`DPCS-DF-006`)
- datasets are reachable from interface inputs (`DPCS-DF-005`)
- data-flow sources are interface inputs or step outputs (`DPCS-DF-007`)
- data-flow destinations are step inputs or interface outputs (`DPCS-DF-008`)
- valid control-flow step dependencies; no opposite-direction conflicts with graph/data flow (`DPCS-CF-004`)
- no duplicate control-flow edges (`DPCS-CF-005`)
- execution capabilities, isolation entries, and external dependency identity/capability
- scheduled mode requires `frequency` or `cron`; event-driven mode requires events
- quality gates require purpose, criteria, and resolvable placement/refs
- failure semantics require scope, triggers, responses; retry response needs retry policy
- lineage dataset/step/contract references resolve against the contract

## Selected diagnostic IDs (0.7 capability matching)

| ID | Meaning |
| --- | --- |
| `DPCS-CAP-001` | Empty capability id in profile |
| `DPCS-CAP-002` | Duplicate capability id in profile |
| `DPCS-CAP-003` | Empty profile identity (missing field is parse `DPCS-PARSE-002`) |
| `DPCS-CAP-004` | Empty profile `dpcsVersion` (omitted/`""` after defaulting; missing-required parse when stricter schemas apply) |
| `DPCS-CAP-005` | Unsupported mandatory capability vs plan/requirements |
| `DPCS-CAP-006` | Profile `dpcsVersion` mismatch warning (vs toolkit and/or plan) |

## Selected diagnostic IDs (0.8 binding)

| ID | Meaning |
| --- | --- |
| `DPCS-BIND-001` | Capability gate failed before orchestrator translation |
| `DPCS-BIND-002` | Unknown binding target |
| `DPCS-BIND-003` | Translation incomplete (for example empty artifact list) |
| `DPCS-BIND-004` | Artifact write failure or unsafe relative path |

## Selected diagnostic IDs (0.6 additions)

| ID | Meaning |
| --- | --- |
| `DPCS-EXE-001` | Empty required capability |
| `DPCS-EXE-003` | Empty external dependency id |
| `DPCS-EXE-004` | Empty external dependency capability |
| `DPCS-EXE-006` | Empty environment values |
| `DPCS-SCH-002` | Scheduled mode missing frequency/cron |
| `DPCS-SCH-003` | Event-driven mode missing events |
| `DPCS-SCH-006` | Comparable ISO-8601 earliest > latest |
| `DPCS-SCH-007` | Free-form earliest/latest not comparable |
| `DPCS-QG-002` | Missing quality criteria |
| `DPCS-QG-004` | Unresolved quality criterion contractRef |
| `DPCS-QG-007` | Unknown step in quality gate placement |
| `DPCS-QG-008` | Empty gate outcome |
| `DPCS-QG-009` | Legacy scope/rule fields |
| `DPCS-FS-003` | Unknown step in failure scope |
| `DPCS-FS-007` | Retry response without meaningful retry semantics |
| `DPCS-FS-008` | Legacy onFailure field |
| `DPCS-LIN-002` | Dataset not present in dataFlow |
| `DPCS-LIN-004` | Unknown producedBy step |
| `DPCS-LIN-010` | Unknown step lineage stepId |
| `DPCS-LIN-016` | Legacy upstream/downstream fields |
| `DPCS-PLN-001` | Plan refused due to validation errors |
| `DPCS-REF-007` | Nested DPCS reference content missing locally |
| `DPCS-REF-008` | Nested DPCS reference content failed to parse |
| `DPCS-REG-016` | Registry artifact content rewrite rejected |

## Selected diagnostic IDs (0.9 additions)

| ID | Meaning |
| --- | --- |
| `DPCS-VER-001` | Invalid SemVer-compatible contract `version` |
| `DPCS-VER-002` | Invalid SemVer-compatible `dpcsVersion` |
| `DPCS-VER-003` | Invalid SemVer-compatible capability profile `dpcsVersion` |
| `DPCS-EXT-001` | Invalid extension namespace |
| `DPCS-EXT-002` | Forbidden extension namespace (profile) |
| `DPCS-EXT-010` | Unrecognized extension preserved (Information) |
| `DPCS-COMPAT-*` | Compatibility analysis findings |
| `DPCS-SEC-*` | Security metadata findings |
| `DPCS-GOV-*` | Governance metadata findings |
| `DPCS-REG-*` | Registry document findings |
| `DPCS-CONF-*` | Conformance claim/profile findings |

Invalid contracts produce diagnostics instead of hard failures where possible.
Planning refuses invalid contracts and emits planning-stage diagnostics.
