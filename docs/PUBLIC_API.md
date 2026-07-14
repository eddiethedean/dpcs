# Public API

## Parsing and validation

```rust
use dpcs::{parse_yaml_file, validate};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let report = validate(&contract);

assert!(report.is_valid());
```

### Performance surfaces (0.12.0)

```rust
use dpcs::{
    parse_yaml_slice, validate, validate_cached, validate_sequential,
    validate_with_context, AnalysisContext, ValidationCache,
};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let _ = validate_sequential(&contract); // always single-threaded

let ctx = AnalysisContext::build(&contract);
let report = validate_with_context(&ctx);

let mut cache = ValidationCache::new();
let _ = validate_cached(&contract, &mut cache);
let _ = validate_cached(&contract, &mut cache); // phases reused
```

Byte-slice parsers: `parse_yaml_slice` / `parse_json_slice`.
Feature `parallel` enables concurrent phases inside `validate` (CLI / `full`).

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
data flow, control flow, execution, scheduling, quality, failure, lineage,
security, governance, and extensions (including namespace rules).

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
Kubernetes are experimental).

Crate-root API: `bind`, `bind_contract`, `parse_target`, `write_bundle`,
`BindContext`, `BindingBundle`, `BindingFile`, `BindingFramework`,
`BindingResult`, `BindingTarget`.

```rust
use dpcs::{
    bind, bind_contract, parse_yaml_file, write_bundle, BindingFramework, BindingResult,
    BindingTarget, CapabilityProfile, PlanResult,
};

assert!(BindingFramework::is_available());

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let profile = CapabilityProfile::from_yaml_file("orchestrator.capabilities.yaml")?;

match bind_contract(&contract, &profile, BindingTarget::Airflow) {
    BindingResult::Ok(bundle) => {
        assert!(!bundle.files.is_empty());
        write_bundle(&bundle, std::path::Path::new("./out")).expect("write artifacts");
    }
    BindingResult::Err {
        diagnostics,
        capability,
    } => {
        assert!(diagnostics.diagnostics.iter().any(|d| {
            d.id == "DPCS-BIND-001" || d.id == "DPCS-PLN-001"
        }));
        let _ = capability; // present when refusal is a capability-gate failure
    }
}
```

`write_bundle` returns `Result<(), ValidationReport>` (not `dpcs::Error`) and
rejects escaping relative paths (`..`, absolute) with `DPCS-BIND-004`. Target
alias `k8s` is accepted for `kubernetes`.

## Compatibility, registry, and conformance (0.9.0)

```rust
use dpcs::{
    compare_contracts, toolkit_claim, validate_claim, validate_conformance_profile,
    validate_registry, CompatibilityResult, ConformanceProfile, Registry,
};

match compare_contracts(&baseline, &candidate) {
    CompatibilityResult::Ok(report) => assert!(report.category.is_compatible()),
    CompatibilityResult::Err { report, .. } => assert!(!report.category.is_compatible()),
}

let registry = Registry::from_file("registry.yaml")?;
assert!(validate_registry(&registry).is_valid());

let profile = ConformanceProfile::from_file("conformance.profile.yaml")?;
assert!(validate_conformance_profile(&profile).is_valid());
assert!(validate_claim(&toolkit_claim()).is_valid());
```

Contract root may declare optional `security` and `governance` blocks.
Extension root keys must use `x-*`, `vendor:name`, or URI-like namespaces.

## Packages, schema, and registry network (0.10.0)

```rust
use dpcs::{validate_package, write_document_schemas};

assert!(validate_package("examples/packages/minimal.dpcspkg").is_valid());
write_document_schemas("schemas")?;
```

Optional features: `cli`, `tui`, `jsonschema`, `registry-client`, `registry-server`, and `full`
(`full` = `cli` + `tui`).
Enable `jsonschema` for `openapi_document` / `write_openapi_documents`.
Enable `registry-client` for `RegistryClient` / `RegistryCache`, and
`registry-server` for `serve` / `serve_listener`. Both registry features share
`PublishRequest` (server no longer depends on the client feature).
Enable `tui` for the interactive inspector (pulled in by `dpcs-cli`).

## Reports and views (0.11.0)

```rust
use dpcs::{
    graph_view_from_contract, inspect_view_from_contract, parse_yaml_file, to_mermaid,
    validation_to_markdown,
};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let inspect = inspect_view_from_contract(&contract);
let graph = graph_view_from_contract(&contract);
let _md = validation_to_markdown(&dpcs::validate(&contract));
let _mmd = to_mermaid(&contract);
let _ = (inspect, graph);
```

## Language bindings (0.10.0)

Python and WebAssembly wrappers live under `bindings/`. Install channels,
package names, and republish workflows are documented in
[`BINDINGS.md`](BINDINGS.md).

[`Error::InvalidDocument`]: ../crates/dpcs/src/error.rs
