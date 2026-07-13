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

COM diagnostics use the `canonicalObjectModel` stage and appear in the standard
validation report:

```rust
let report = contract.validate();
for diagnostic in &report.diagnostics {
    if diagnostic.category == "canonicalObjectModel" {
        eprintln!("{}: {}", diagnostic.id, diagnostic.message);
    }
}
```

## Future API

```rust
let plan = dpcs::plan(&contract)?;
let binding = dpcs::bind(&plan, airflow_capabilities)?;
```

Do not implement binding yet.
