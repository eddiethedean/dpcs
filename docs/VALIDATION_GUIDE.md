# Validation Guide

Validation is deterministic and phase-based. It returns a `ValidationReport`
and does not panic on invalid contracts. Phases always complete and findings are
accumulated, then sorted deterministically.

## Phases (0.5.0)

1. Document — unsupported `dpcsVersion` warnings (`DPCS-DOC-002`)
2. Canonical Object Model — identity, uniqueness, interface completeness, reserved extension keys
3. Structural — empty step `type` (`DPCS-STR-003`), empty step port ids (`DPCS-STR-001`)
4. Graph — edges, cycles, duplicate edges, unreachable steps, entry/exit integrity
5. References — resolvable contract / transform / port references
6. Data Flow — endpoints, dataset identity, wiring, unreachable datasets
7. Control Flow — step endpoints, conflicting deps, duplicate control edges
8. Quality / Failure — identity owned by COM; rule semantics deferred to ROADMAP 0.6.0
9. Extensions — reserved root collisions are COM-012; namespace rules deferred to 0.9.0

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
- valid control-flow step dependencies; no opposite-direction conflicts with graph/data flow (`DPCS-CF-004`)
- no duplicate control-flow edges (`DPCS-CF-005`)

## Selected diagnostic IDs (0.5 additions)

| ID | Meaning |
| --- | --- |
| `DPCS-STR-001` | Empty step port identifier |
| `DPCS-REF-005` | Unresolved `transformRef` |
| `DPCS-REF-006` | Unresolved step-port `contractRef` |
| `DPCS-DF-004` | Missing data-flow dataset identity |
| `DPCS-DF-005` | Unreachable dataset |
| `DPCS-DF-006` | Unsatisfied step input / interface output |
| `DPCS-CF-004` | Conflicting control vs graph/data dependency |
| `DPCS-CF-005` | Duplicate control-flow edge |

Invalid contracts produce diagnostics instead of hard failures where possible.
