# Public API Draft

Initial Rust API:

```rust
use dpcs::{parse_yaml_file, validate};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let report = validate(&contract);

assert!(report.is_valid());
```

Object-oriented API:

```rust
use dpcs::PipelineContract;

let contract = PipelineContract::from_yaml_file("pipeline.dpcs.yaml")?;
let report = contract.validate();
```

Future API:

```rust
let plan = dpcs::plan(&contract)?;
let binding = dpcs::bind(&plan, airflow_capabilities)?;
```

Do not implement binding yet.
