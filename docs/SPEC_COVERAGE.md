# SPEC Coverage Matrix (0.13.0)

Living inventory of normative requirements from
[`SPEC.md`](https://github.com/eddiethedean/dpcs/blob/main/SPEC.md) versus
the `dpcs` toolkit. Status values:

| Status | Meaning |
| --- | --- |
| **Implemented** | Toolkit-actionable SHALL satisfied and covered by tests |
| **Gap** | Toolkit-actionable SHALL not yet satisfied (must be zero for 0.13 ship) |
| **Spec-OOS** | Explicitly out of scope of DPCS (Ch 1 §4) or project NON_GOALS |
| **Process-advisory** | Organizational SHOULD/SHALL in Ch 24/25 that apply to operators/standards bodies, not library behavior |
| **Informative** | Appendix / recommended practice |

Diagnostic IDs are implementation-defined; see [`diagnostics.catalog.json`](diagnostics.catalog.json).

## Chapter summary

| Chapter | SHALLs (approx) | Primary modules | Status |
| --- | --- | --- | --- |
| 1 Introduction | 6 | docs / NON_GOALS | Spec-OOS where Ch 1 §4 applies; else Implemented |
| 2 Core Concepts | 10 | `model/`, guides | Implemented |
| 3 Canonical Object Model | 21 | `model/`, `parser/`, `validation/com` | Implemented |
| 4 Pipeline Interface | 19 | `model/interface`, `validation/com` | Implemented |
| 5 Pipeline Graph | 23 | `validation/graph`, `resolve/`, `plan/` | Implemented (nested parent–child + recursive resolve) |
| 6 Pipeline Steps | 22 | `model/step`, `resolve/`, `plan/` | Implemented (nested contracts + interface port lists on plan) |
| 7 Contract References | 19 | `resolve/`, `validation/references`, `plan/` | Implemented (`plan()` deep-resolves by default) |
| 8 Data Flow | 20 | `validation/data_flow` | Implemented |
| 9 Control Flow | 21 | `validation/control_flow` | Implemented |
| 10 Execution Requirements | 16 | `validation/execution` | Implemented |
| 11 Scheduling Intent | 17 | `validation/scheduling`, binding scaffolds | Implemented |
| 12 Quality Gates | 18 | `validation/quality`, binding scaffolds | Implemented |
| 13 Failure Semantics | 16 | `validation/failure`, binding scaffolds | Implemented |
| 14 Pipeline Lineage | 16 | `validation/lineage`, plan provenance | Implemented |
| 15 Pipeline Plan | 22 | `plan/` | Implemented (resolve-before-plan) |
| 16 Capability Model | 17 | `capabilities/` | Implemented |
| 17 Orchestrator Binding | 15 | `binding/` | Implemented (**scaffold bundle + `dpcs_semantics.json`**; not production operators). Temporal/Kubernetes labeled experimental |
| 18 Diagnostics | 20 | `diagnostics/` | Implemented |
| 19 Compatibility | 16 | `compatibility/` | Implemented (`compare_contracts` / `compare_plans` asserted in Appendix E) |
| 20 Versioning | 23 | `model/versioning` | Implemented |
| 21 Extensibility | 18 | `validation/extensions` | Implemented |
| 22 Registries | 15 | `model/registry`, `registry_net/` | Implemented (reference HTTP; **id@version content immutable**, `DPCS-REG-016` / HTTP 409) |
| 23 Conformance | 17 | `conformance/`, `tests/conformance*` | Implemented (`CompleteImplementation` claimed; golden COM YAML↔JSON interchange via round_trip) |
| 24 Security | 12 | `validation/security` | Process-advisory ops; metadata Implemented |
| 25 Governance | 12 | `validation/governance` | Process-advisory ops; metadata Implemented |
| 26 Appendices | 0 | Appendix E → conformance suite | Informative |

## Toolkit-actionable Gap tracking (0.13)

| ID | Requirement | Status | Resolution |
| --- | --- | --- | --- |
| G-01 | Ch 7 resolve Contract References before planning | Closed | `plan()` / `bind_contract()` use `ResolveOptions::default_for_planning()`; document-relative paths use `from_document_path` |
| G-02 | Ch 5/6 nested pipeline identity + parent–child | Closed | Recursive resolve (depth/cycle guards), `NestedPlanPipeline` ports/stepOrder/children, provenance |
| G-03 | Ch 17 translate scheduling / QG / FS / execution | Closed | Structured `dpcs_semantics.json` on every bind target (scaffold + sidecar equivalence) |
| G-04 | Appendix E suite | Closed | `tests/conformance*.rs` + `make conformance` with resolve/plan/`compare_plans`/nesting asserts |
| G-05 | Ch 22/23 registry immutability | Closed | HTTP publish rejects content-changing republish of same id@version (`DPCS-REG-016`) |

Open Gap rows for toolkit-actionable SHALLs: **none**.

## Spec-OOS (does not count against 100%)

- Dataset schemas / transform semantics / engine internals (SPEC Ch 1 §4)
- Execution / scheduling runtimes, data movement (NON_GOALS)
- Production Airflow/Dagster/Temporal workers (ADR-0003 / NON_GOALS)
- ODCS / DTCS validation internals (NON_GOALS)
- Lifetime-parameterized COM, multi-document TUI (NON_GOALS)
- Multi-implementation interoperability suites beyond same-RI golden COM fixtures (SPEC Ch 23 MAY)

## Process-advisory (metadata only)

- Ch 24 operational authn/z, crypto verify, audit platforms
- Ch 25 standards-org publication roles and process

## Conformance claim

`implemented_levels()` / `toolkit_claim()` include `CompleteImplementation` when this
matrix has zero Gap rows and `make conformance` is green.
