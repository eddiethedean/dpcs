# Project Goal

Build `dpcs`, a Rust-first implementation of the Data Pipeline Contract Standard.

The crate implements foundational DPCS specification logic through ROADMAP 0.11.0:

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
- Compatibility analysis
- Versioning / extensibility / registries (document model + reference network API)
- Conformance claims and Appendix E suite
- Security and governance metadata
- Pipeline packages (`.dpcspkg`)
- JSON Schema / OpenAPI helpers
- Python and WASM bindings (PyPI `dpcs`, npm `@eddiethedean/dpcs`, Wasmer `eddiethedean/dpcs`)
- Report renderers (Markdown / HTML / Mermaid / DOT) and rich CLI formats
- Interactive TUI inspector
- CLI

See [`BINDINGS.md`](BINDINGS.md) for install and release channel names.

Future milestones:

- Performance work (0.12)
- Full reference-implementation / conformance completion (0.13)
