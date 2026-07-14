# Planning and binding

## Planning

`plan` always deep-resolves Contract References before planning (SPEC Ch 7)
using [`ResolveOptions::default_for_planning`](PUBLIC_API.md) (process CWD).
Prefer `plan_with_resolve` + `ResolveOptions::from_document_path` when locations
are relative to a contract file (as the CLI does).

A `PipelinePlan` is produced only when validation and nested DPCS resolution
succeed. Nested contracts appear on `PipelinePlan.nested` with:

- `contractId` / `contractVersion` / `dpcsVersion`
- interface `inputPorts` / `outputPorts`
- nested `stepOrder`
- recursive `children`
- lineage provenance parent/child links

Unresolved nested DPCS refs fail planning (`DPCS-REF-007` / `DPCS-REF-008` →
`DPCS-PLN-001`). Companion ODCS/DTCS locations may be external.

```rust
use dpcs::{plan, plan_with_resolve, ResolveOptions};

// CWD-relative resolve (library default)
let _ = plan(&contract);

// Document-relative resolve (CLI default)
let opts = ResolveOptions::from_document_path("parent.dpcs.yaml");
match plan_with_resolve(&contract, Some(&opts)) {
    dpcs::PlanResult::Ok(plan) => {
        assert!(!plan.step_order.is_empty());
        let _ = &plan.nested;
    }
    dpcs::PlanResult::Err(report) => eprintln!("{report:?}"),
}
```

See also [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md) (G-01 / G-02).

## Capabilities

`evaluate` matches plan demand (`requiredCapabilities` + external dependency
capabilities) against a `CapabilityProfile` without mutating the plan.

## Binding

`bind` / `bind_contract` / `bind_contract_with_resolve` emit platform scaffold
artifacts **plus** `dpcs_semantics.json` encoding scheduling, quality gates,
failure semantics, execution requirements, nested pipelines, and dependencies
(Ch 17 scaffold bundle equivalence per ADR-0003).

Temporal and Kubernetes adapters remain labeled experimental; all five targets
share the same fidelity checklist. Production operator libraries are out of
scope ([`NON_GOALS.md`](NON_GOALS.md)).

```bash
dpcs bind examples/with_execution.dpcs.yaml \
  --profile examples/orchestrator.capabilities.yaml \
  --target airflow --out /tmp/dpcs-bind
# includes dags/*.py and dpcs_semantics.json
```
