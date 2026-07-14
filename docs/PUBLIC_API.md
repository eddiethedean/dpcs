# Public API

## Parsing and validation

```rust
use dpcs::{parse_yaml_file, validate};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let report = validate(&contract);

assert!(report.is_valid());
```

Parse failures return [`Error::InvalidDocument`] with Parse-stage diagnostics:

```rust
use dpcs::{parse_yaml, DiagnosticStage, Error};

match parse_yaml("id: only-id\n") {
    Ok(contract) => { /* ... */ }
    Err(Error::InvalidDocument { report }) => {
        assert!(report.diagnostics.iter().all(|d| d.stage == DiagnosticStage::Parse));
    }
    Err(err) => panic!("unexpected error: {err}"),
}
```

## Serialization (0.3.0)

```rust
use dpcs::{parse_yaml_file, to_json, to_yaml};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let yaml = to_yaml(&contract)?;
let json = to_json(&contract)?;
```

Object-oriented helpers:

```rust
use dpcs::PipelineContract;

let contract = PipelineContract::from_yaml_file("pipeline.dpcs.yaml")?;
let yaml = contract.to_yaml_str()?;
let json = contract.to_json_str()?;
```

Format dispatch by file extension:

```rust
use dpcs::{parse_file, to_file};

let contract = parse_file("pipeline.dpcs.yaml")?;
to_file(&contract, "pipeline.copy.yaml")?;
```

## Object-oriented API

```rust
use dpcs::PipelineContract;

let contract = PipelineContract::from_yaml_file("pipeline.dpcs.yaml")?;
let report = contract.validate();
```

## Identity and COM types (0.2.0)

```rust
use dpcs::{
    ExtensionValue, IdentityCatalog, ObjectId, ObjectKind, ObjectPath,
    PipelineIdentity, PipelineInterface, InterfacePort, Metadata,
};

let identity: PipelineIdentity = contract.identity();
let catalog: IdentityCatalog = contract.identity_catalog();

assert!(identity.is_complete());
assert!(catalog.get_by_path("pipeline").is_some());
```

## COM invariant validation

COM diagnostics use the `canonicalObjectModel` stage and category:

```rust
let report = contract.validate();
for diagnostic in &report.diagnostics {
    if diagnostic.category == "canonicalObjectModel" {
        eprintln!("{}: {}", diagnostic.id, diagnostic.message);
    }
}
```

## Graph analysis (0.4.0)

```rust
use dpcs::{parse_yaml_file, DependencyGraph};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let graph = DependencyGraph::from_contract(&contract);

let order = graph.topological_order()?;
let deps = graph.dependencies("normalize_customer");
let unreachable = graph.unreachable_steps(&contract);
let duplicates = DependencyGraph::duplicate_edges(&contract);
```

## Validation engine (0.5.0–0.6.0)

```rust
use dpcs::{parse_yaml_file, unreachable_datasets, unsatisfied_ports, validate};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let report = validate(&contract);
assert!(report.is_valid());

let missing_ports = unsatisfied_ports(&contract);
let orphan_datasets = unreachable_datasets(&contract);
```

Phase-based validation covers document, COM, structural, graph, references,
data flow, control flow, execution, scheduling, quality, failure, and lineage.
Extension namespace rules remain deferred to ROADMAP 0.9.0.

## Planning (0.6.0)

`plan` produces a full `PipelinePlan` only from a successfully validated
contract:

```rust
use dpcs::{parse_yaml_file, plan, PlanResult};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
match plan(&contract) {
    PlanResult::Ok(planned) => {
        assert!(!planned.step_order.is_empty() || contract.steps.is_empty());
        let _ = planned.execution;
        let _ = &planned.scheduling;
        let _ = &planned.quality_gates;
        let _ = &planned.failure_semantics;
        let _ = &planned.lineage;
    }
    PlanResult::Err(report) => {
        assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PLN-001"));
    }
}

assert!(dpcs::binding::BindingFramework::is_available());
```

## Capability evaluation (0.7.0)

Match a planned pipeline (or raw `ExecutionRequirements`) against an orchestrator
profile without mutating the plan:

```rust
use dpcs::{
    evaluate, parse_yaml_file, plan, CapabilityProfile, CapabilityResult, PlanResult,
};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let profile = CapabilityProfile::from_yaml_file("orchestrator.capabilities.yaml")?;
let PlanResult::Ok(planned) = plan(&contract) else {
    panic!("contract must plan");
};

match evaluate(&planned, &profile) {
    CapabilityResult::Ok(report) => {
        assert!(report.missing_mandatory.is_empty());
    }
    CapabilityResult::Err { report, diagnostics } => {
        assert!(!report.missing_mandatory.is_empty());
        assert!(diagnostics.diagnostics.iter().any(|d| d.id == "DPCS-CAP-005"));
    }
}
```

Demand matched against a profile is `requiredCapabilities` plus
`externalDependencies[].capability`. Environment `softwareCapabilities` and
`isolation` are not treated as orchestrator capability ids.

`OrchestratorCapabilities` remains a deprecated name alias for
`CapabilityProfile`. Prefer `CapabilityProfile`.

## Orchestrator binding (0.8.0)

Bind a planned pipeline to an orchestrator target after a successful capability
match. Adapters emit scaffold artifacts (Airflow/Dagster/Prefect; Temporal and
Kubernetes are experimental):

```rust
use dpcs::{
    bind, parse_yaml_file, plan, write_bundle, BindingResult, BindingTarget,
    CapabilityProfile, PlanResult,
};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let profile = CapabilityProfile::from_yaml_file("orchestrator.capabilities.yaml")?;
let PlanResult::Ok(planned) = plan(&contract) else {
    panic!("contract must plan");
};

match bind(&planned, &profile, BindingTarget::Airflow) {
    BindingResult::Ok(bundle) => {
        assert!(!bundle.files.is_empty());
        write_bundle(&bundle, std::path::Path::new("./out"))?;
    }
    BindingResult::Err(report) => {
        assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-BIND-001"));
    }
}
```

[`Error::InvalidDocument`]: ../src/error.rs
