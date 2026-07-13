# Public API

## Parsing and validation

```rust
use dpcs::{parse_yaml_file, validate};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let report = validate(&contract);

assert!(report.is_valid());
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

## Planning skeleton

A minimal planner exists today; orchestrator binding does not:

```rust
let plan = dpcs::plan::plan(&contract);
assert!(!dpcs::binding::BindingFramework::is_available());
```

Full planning semantics and binding adapters are roadmap items 0.6–0.8.
