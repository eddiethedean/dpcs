# Getting started

Install the CLI from this workspace (or crates.io once published):

```bash
cargo install --path crates/dpcs-cli
# cargo install dpcs-cli --version 0.13.0
dpcs version --json
```

Language packages: `pip install dpcs`, `npm install @eddiethedean/dpcs`
(see [`BINDINGS.md`](BINDINGS.md)).

## Validate and explore

```bash
dpcs validate examples/minimal.dpcs.yaml
dpcs inspect examples/minimal.dpcs.yaml --format markdown
dpcs graph examples/with_execution.dpcs.yaml --format mermaid
dpcs conformance validate examples/conformance.profile.yaml
```

CLI validate / plan / bind resolve nested DPCS refs relative to the **document
directory**. Companion ODCS/DTCS files need not exist locally.

## Library

```rust
use dpcs::{parse_yaml_file, plan, plan_with_resolve, validate_resolved, ResolveOptions};

let path = "pipeline.dpcs.yaml";
let contract = parse_yaml_file(path)?;

// Document-relative resolve (recommended when locations are beside the file)
let opts = ResolveOptions::from_document_path(path);
let report = validate_resolved(&contract, &opts);
assert!(report.is_valid());
let _plan = plan_with_resolve(&contract, Some(&opts));

// Bare plan() resolves using the process current directory
let _ = plan(&contract);
```

## Next

| Topic | Page |
| --- | --- |
| Public API + stability | [`PUBLIC_API.md`](PUBLIC_API.md) |
| SPEC completeness | [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md) |
| Planning / binding | [`PLANNING.md`](PLANNING.md) |
| Conformance suite | [`CONFORMANCE.md`](CONFORMANCE.md) |
| CLI flags | [`CLI_SPEC.md`](CLI_SPEC.md) |
