# Planning and binding

## Planning

`plan` / `plan_with_resolve` produce a `PipelinePlan` only when validation (and
optional deep reference resolution) succeed. Nested DPCS contracts appear on
`PipelinePlan.nested` with lineage provenance parent/child links.

```rust
use dpcs::{plan_with_resolve, ResolveOptions};

let opts = ResolveOptions::from_document_path("parent.dpcs.yaml");
match plan_with_resolve(&contract, Some(&opts)) {
    dpcs::PlanResult::Ok(plan) => assert!(!plan.step_order.is_empty()),
    dpcs::PlanResult::Err(report) => eprintln!("{report:?}"),
}
```

## Capabilities

`evaluate` matches plan demand (`requiredCapabilities` + external dependency
capabilities) against a `CapabilityProfile` without mutating the plan.

## Binding

`bind` / `bind_contract_with_resolve` emit scaffold artifacts plus
`dpcs_semantics.json` encoding scheduling, quality gates, failure semantics,
execution requirements, nested pipelines, and dependencies.

Temporal and Kubernetes adapters remain labeled experimental; all five targets
share the same fidelity checklist. Production operator libraries are out of
scope ([`NON_GOALS.md`](NON_GOALS.md)).
