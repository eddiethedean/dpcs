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
- Pipeline Plan generation
- Orchestrator Capability Model (profiles and matching)
- Orchestrator Binding (scaffold adapters)
- CLI

Future milestones:

- Deeper / production-grade orchestrator backends
- Python bindings
- WASM bindings
