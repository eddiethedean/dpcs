# DPCS

[![CI](https://github.com/eddiethedean/dpcs/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/dpcs/actions/workflows/ci.yml)
[![Release](https://github.com/eddiethedean/dpcs/actions/workflows/release.yml/badge.svg)](https://github.com/eddiethedean/dpcs/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/dpcs.svg)](https://crates.io/crates/dpcs)
[![Rust API](https://img.shields.io/docsrs/dpcs?label=rust%20api)](https://docs.rs/dpcs)
[![Guides](https://img.shields.io/badge/guides-readthedocs.io-blue?logo=readthedocs&logoColor=white)](https://dpcs.readthedocs.io/en/latest/)
[![PyPI](https://img.shields.io/pypi/v/dpcs.svg)](https://pypi.org/project/dpcs/)
[![npm](https://img.shields.io/npm/v/@eddiethedean/dpcs.svg)](https://www.npmjs.com/package/@eddiethedean/dpcs)
[![Wasmer](https://img.shields.io/badge/wasmer-eddiethedean%2Fdpcs-blue)](https://wasmer.io/eddiethedean/dpcs)
[![MSRV](https://img.shields.io/crates/msrv/dpcs)](https://crates.io/crates/dpcs)
[![License](https://img.shields.io/crates/l/dpcs.svg)](#license)

Reference implementation of the **Data Pipeline Contract Standard (DPCS)**.

`dpcs` is a Rust-first toolkit for parsing, inspecting, and validating portable,
contract-first data pipeline definitions. The full specification lives in
[`SPEC.md`](SPEC.md) and is the authoritative source of truth.

```text
DPCS Document -> Parser -> COM -> Validator -> Pipeline Plan -> Capability Evaluation -> Orchestrator Binding
```

Binding adapters emit scaffold artifacts for Airflow, Dagster, Prefect, Temporal,
and Kubernetes. Execution runtimes remain out of scope. See [`ROADMAP.md`](ROADMAP.md).

## Status

| Item | Value |
| --- | --- |
| Crate version | `0.13.1` |
| Spec version | `1.0.0-draft` |
| Language | Rust 2021 (MSRV 1.88) |
| License | Apache-2.0 OR MIT |
| Guides | [dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/) |
| API docs | [docs.rs/dpcs](https://docs.rs/dpcs) |
| Release focus | Reference implementation: full SPEC coverage, conformance, stable API (ROADMAP 0.13.0) |

## Quick start

### Install

```bash
cargo install --path crates/dpcs-cli
# or published packages:
# cargo install dpcs-cli --version 0.13.1
# pip install dpcs
# npm install @eddiethedean/dpcs
```

Language package names and republish notes: [`docs/BINDINGS.md`](docs/BINDINGS.md).
Guides and CLI docs: [dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/).

### Validate a pipeline contract

```bash
dpcs validate examples/minimal.dpcs.yaml
dpcs validate examples/minimal.dpcs.yaml --json
dpcs validate examples/minimal.dpcs.yaml --strict
dpcs compatibility examples/compatibility/baseline.dpcs.yaml examples/compatibility/candidate_compatible.dpcs.yaml
dpcs registry validate examples/registry.yaml
dpcs package validate examples/packages/minimal.dpcspkg
dpcs schema json --out schemas
dpcs conformance validate examples/conformance.profile.yaml
dpcs version --json
```

### Inspect and explore

```bash
dpcs inspect examples/minimal.dpcs.yaml
dpcs diagnostics examples/minimal.dpcs.yaml --json
dpcs graph examples/minimal.dpcs.yaml
dpcs capabilities examples/orchestrator.capabilities.yaml --plan examples/with_execution.dpcs.yaml
dpcs bind examples/with_execution.dpcs.yaml --profile examples/orchestrator.capabilities.yaml --target airflow
dpcs version
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | `validate`/`diagnostics`: valid; `capabilities`: match ok; `bind`: success; `compatibility`: compatible; `registry`/`conformance` validate: ok; `inspect`/`graph`: successful parse |
| `1` | Validation, capability, binding, compatibility, registry, or conformance errors |
| `2` | Parse or I/O failure |

## Library usage

```rust
use dpcs::{parse_yaml_file, validate};

fn main() -> dpcs::Result<()> {
    let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
    let report = validate(&contract);

    if report.is_valid() {
        println!("contract `{}` is valid", contract.id);
    } else {
        for diagnostic in &report.diagnostics {
            eprintln!("{}: {}", diagnostic.id, diagnostic.message);
        }
    }

    Ok(())
}
```

Object-oriented style:

```rust
use dpcs::PipelineContract;

let contract = PipelineContract::from_yaml_file("pipeline.dpcs.yaml")?;
let report = contract.validate();
assert!(report.is_valid());

let yaml = contract.to_yaml_str()?;
let json = contract.to_json_str()?;
```

Graph analysis and planning (deep resolve on `plan()`; prefer
`plan_with_resolve` + `ResolveOptions::from_document_path` for file-relative nests):

```rust
use dpcs::{
    parse_yaml_file, plan, plan_with_resolve, DependencyGraph, PlanResult, ResolveOptions,
};

let path = "pipeline.dpcs.yaml";
let contract = parse_yaml_file(path)?;
let graph = DependencyGraph::from_contract(&contract);

if let Ok(order) = graph.topological_order() {
    println!("step order: {:?}", order);
}

let opts = ResolveOptions::from_document_path(path);
match plan_with_resolve(&contract, Some(&opts)) {
    PlanResult::Ok(planned) => {
        println!("plan steps: {:?}", planned.step_order);
        println!("nested: {}", planned.nested.len());
    }
    PlanResult::Err(report) => eprintln!("planning refused: {} errors", report.error_count()),
}

// Bare plan() also deep-resolves (paths relative to process CWD)
let _ = plan(&contract);
```

Capability evaluation:

```rust
use dpcs::{evaluate, CapabilityProfile, CapabilityResult, PlanResult};

let profile = CapabilityProfile::from_yaml_file("orchestrator.capabilities.yaml")?;
if let PlanResult::Ok(planned) = plan(&contract) {
    match evaluate(&planned, &profile) {
        CapabilityResult::Ok(report) => println!("satisfied: {:?}", report.satisfied),
        CapabilityResult::Err { diagnostics, .. } => {
            eprintln!("capability errors: {}", diagnostics.error_count())
        }
    }
}
```

Orchestrator binding (scaffolds + `dpcs_semantics.json`):

```rust
use dpcs::{bind, BindingResult, BindingTarget};

if let PlanResult::Ok(planned) = plan(&contract) {
    match bind(&planned, &profile, BindingTarget::Airflow) {
        BindingResult::Ok(bundle) => println!("files: {}", bundle.files.len()),
        BindingResult::Err { diagnostics, .. } => {
            eprintln!("bind failed: {}", diagnostics.error_count())
        }
    }
}
```

Compatibility, registry, and conformance (0.9.0):

```rust
use dpcs::{
    compare_contracts, toolkit_claim, validate_claim, validate_registry, CompatibilityResult,
    Registry,
};

match compare_contracts(&baseline, &candidate) {
    CompatibilityResult::Ok(report) => println!("category: {}", report.category),
    CompatibilityResult::Err { report, .. } => eprintln!("incompatible: {}", report.category),
}

let registry = Registry::from_yaml_str(yaml)?;
assert!(validate_registry(&registry).is_valid());
assert!(validate_claim(&toolkit_claim()).is_valid());
```

## Repository layout

```text
.
├── Cargo.toml              # Workspace (excludes bindings/python)
├── SPEC.md                 # Authoritative DPCS specification
├── ROADMAP.md              # Release plan
├── crates/
│   ├── dpcs/               # Core library
│   └── dpcs-cli/           # CLI binary
├── bindings/
│   ├── python/             # PyO3 / maturin → PyPI `dpcs`
│   └── wasm/               # wasm-bindgen → npm + Wasmer
├── schemas/                # Generated JSON Schema + OpenAPI
├── examples/               # Example contracts and profiles
├── docs/                   # Design and contributor guides
└── adr/                    # Architecture decision records
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Useful docs:

- [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md)
- [`docs/PUBLIC_API.md`](docs/PUBLIC_API.md)
- [`docs/PLANNING.md`](docs/PLANNING.md)
- [`docs/SPEC_COVERAGE.md`](docs/SPEC_COVERAGE.md)
- [`docs/CONFORMANCE.md`](docs/CONFORMANCE.md)
- [`docs/CLI_SPEC.md`](docs/CLI_SPEC.md)
- [`docs/BINDINGS.md`](docs/BINDINGS.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- Guides index: [dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/)

## Design principles

- **SPEC.md is authoritative.** Implementation follows the specification, not the other way around.
- **Contract-first.** Pipelines are portable declarations, not engine-specific DAGs.
- **Deterministic diagnostics.** Validation returns structured findings, never panics on invalid input.
- **Incremental delivery.** Each `0.x` release completes a coherent slice of the roadmap.

## Relationship to ODCS and DTCS

```text
ODCS  -> what data is
DTCS  -> how data changes
DPCS  -> how transformations compose into pipelines
```

DPCS references ODCS and DTCS artifacts through contract references. It does not
re-implement those standards.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`ROADMAP.md`](ROADMAP.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
