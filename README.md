# DPCS

[![CI](https://github.com/eddiethedean/dpcs/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/dpcs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/dpcs.svg)](https://crates.io/crates/dpcs)
[![docs.rs](https://img.shields.io/docsrs/dpcs)](https://docs.rs/dpcs)
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
| Crate version | `0.10.0` |
| Spec version | `1.0.0-draft` |
| Language | Rust 2021 (MSRV 1.85) |
| License | Apache-2.0 OR MIT |
| Release focus | Ecosystem: bindings, schemas, registry client, packages (ROADMAP 0.10.0) |

## Quick start

### Install

```bash
cargo install --path crates/dpcs-cli
# or, after crates.io publish:
# cargo install dpcs-cli --version 0.10.0
# pip install dpcs
# npm install @dpcs/dpcs-wasm   # after publish; see bindings/wasm
```

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

Graph analysis (0.4.0) and planning (0.6.0):

```rust
use dpcs::{parse_yaml_file, plan, DependencyGraph, PlanResult};

let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
let graph = DependencyGraph::from_contract(&contract);

if let Ok(order) = graph.topological_order() {
    println!("step order: {:?}", order);
}
if let Some(cycle) = graph.find_cycle() {
    eprintln!("cycle: {:?}", cycle);
}

match plan(&contract) {
    PlanResult::Ok(planned) => println!("plan steps: {:?}", planned.step_order),
    PlanResult::Err(report) => eprintln!("planning refused: {} errors", report.error_count()),
}
```

Capability evaluation (0.7.0):

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

Orchestrator binding (0.8.0):

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
├── SPEC.md                 # Authoritative DPCS specification
├── ROADMAP.md              # Release plan
├── src/
│   ├── model/              # Canonical Object Model
│   ├── parser/             # YAML and JSON parsers
│   ├── validation/         # Phase-based validation
│   ├── diagnostics/        # Deterministic diagnostics
│   ├── plan/               # Deterministic Pipeline Plan IR
│   ├── capabilities/       # Capability profiles and matcher
│   ├── binding/            # Orchestrator binding scaffolds
│   ├── compatibility/      # Compatibility analysis
│   ├── conformance/        # Conformance claims and profiles
│   └── cli/                # CLI implementation
├── examples/               # Example contracts and profiles
├── tests/fixtures/         # Valid and invalid fixtures
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

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/CRATE_LAYOUT.md`](docs/CRATE_LAYOUT.md)
- [`docs/PUBLIC_API.md`](docs/PUBLIC_API.md)
- [`docs/CLI_SPEC.md`](docs/CLI_SPEC.md)
- [`docs/TESTING_PLAN.md`](docs/TESTING_PLAN.md)
- [`docs/IMPLEMENTATION_PHASES.md`](docs/IMPLEMENTATION_PHASES.md)
- [`docs/NON_GOALS.md`](docs/NON_GOALS.md)

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
