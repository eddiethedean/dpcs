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

`plan::plan` uses topological ordering when the dependency graph is acyclic.

## Planning skeleton

A minimal planner exists today; orchestrator binding does not:

```rust
let plan = dpcs::plan::plan(&contract);
assert!(!dpcs::binding::BindingFramework::is_available());
```

Full planning semantics and binding adapters are roadmap items 0.6–0.8.

[`Error::InvalidDocument`]: ../src/error.rs
