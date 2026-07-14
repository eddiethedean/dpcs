# Getting started

Install the CLI from this workspace:

```bash
cargo install --path crates/dpcs-cli
dpcs version --json
```

Validate and inspect a contract:

```bash
dpcs validate examples/minimal.dpcs.yaml
dpcs inspect examples/minimal.dpcs.yaml --format markdown
dpcs graph examples/with_execution.dpcs.yaml --format mermaid
```

Library entry points:

```rust
use dpcs::{parse_yaml_file, plan_with_resolve, validate_resolved, ResolveOptions};

let path = "pipeline.dpcs.yaml";
let contract = parse_yaml_file(path)?;
let opts = ResolveOptions::from_document_path(path);
let report = validate_resolved(&contract, &opts);
assert!(report.is_valid());
let _plan = plan_with_resolve(&contract, Some(&opts));
```

Next: [`PUBLIC_API.md`](PUBLIC_API.md), [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md),
[`CONFORMANCE.md`](CONFORMANCE.md).
