# Validation Guide

Validation is deterministic and phase-based. It returns a `ValidationReport`
and does not panic on invalid contracts. Phases always complete and findings are
accumulated, then sorted deterministically.

## Phases (0.6.0)

1. Document — unsupported `dpcsVersion` warnings (`DPCS-DOC-002`)
2. Canonical Object Model — identity, uniqueness, interface completeness, reserved extension keys
3. Structural — empty step `type` (`DPCS-STR-003`), empty step port ids (`DPCS-STR-001`)
4. Graph — edges, cycles, duplicate edges, unreachable steps, entry/exit integrity
5. References — resolvable contract / transform / port references
6. Data Flow — endpoints, dataset identity, wiring, unreachable datasets
7. Control Flow — step endpoints, conflicting deps, duplicate control edges
8. Execution — capability/dependency/resource completeness (`DPCS-EXE-*`)
9. Scheduling — modes, events, timing consistency (`DPCS-SCH-*`)
10. Quality — criteria, outcomes, placement, contract refs (`DPCS-QG-*`)
11. Failure — scope, triggers, responses, retry (`DPCS-FS-*`)
12. Lineage — dataset/step/contract provenance refs (`DPCS-LIN-*`)
13. Extensions — reserved root collisions are COM-012; namespace rules deferred to 0.9.0

Quality/failure **identity** (empty/duplicate ids) remains COM-owned
(`DPCS-COM-004` / `DPCS-COM-005`).

Capability matching against orchestrator profiles is deferred to ROADMAP 0.7.0.

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

## Selected diagnostic IDs (0.6 additions)

| ID | Meaning |
| --- | --- |
| `DPCS-EXE-001` | Empty required capability |
| `DPCS-EXE-003` | Empty external dependency id |
| `DPCS-EXE-004` | Empty external dependency capability |
| `DPCS-SCH-002` | Scheduled mode missing frequency/cron |
| `DPCS-SCH-003` | Event-driven mode missing events |
| `DPCS-SCH-006` | earliest > latest timing constraint |
| `DPCS-QG-002` | Missing quality criteria |
| `DPCS-QG-004` | Unresolved quality criterion contractRef |
| `DPCS-QG-007` | Unknown step in quality gate placement |
| `DPCS-FS-003` | Unknown step in failure scope |
| `DPCS-FS-007` | Retry response without retry semantics |
| `DPCS-LIN-004` | Unknown producedBy step |
| `DPCS-LIN-010` | Unknown step lineage stepId |
| `DPCS-PLN-001` | Plan refused due to validation errors |

Invalid contracts produce diagnostics instead of hard failures where possible.
Planning refuses invalid contracts and emits planning-stage diagnostics.
