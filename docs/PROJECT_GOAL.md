# Project Goal

Build `dpcs`, a Rust-first implementation of the Data Pipeline Contract Standard.

The crate should implement the foundational DPCS specification logic:

- Canonical Object Model
- YAML and JSON parsing
- Validation phases
- Diagnostics
- Pipeline graph validation
- Contract reference model
- Data Flow and Control Flow validation
- Pipeline Plan skeleton
- Orchestrator Capability Model skeleton
- CLI

Future milestones:

- Pipeline Plan generation
- Capability evaluation
- Orchestrator Binding
- Airflow/Dagster/Prefect/Temporal/Kubernetes backends
- Python bindings
- WASM bindings
