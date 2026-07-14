# Project Goal

Build `dpcs`, a Rust-first implementation of the Data Pipeline Contract Standard.

The crate implements foundational DPCS specification logic through ROADMAP 0.13.0:

- Canonical Object Model
- YAML and JSON parsing
- Validation phases
- Diagnostics
- Pipeline graph validation
- Contract reference model (deep resolve + nested pipelines)
- Data Flow and Control Flow validation
- Pipeline Plan generation
- Orchestrator Capability Model (profiles and matching)
- Orchestrator Binding (scaffold adapters + structured semantics)
- Compatibility analysis
- Versioning / extensibility / registries (document model + reference network API)
- Conformance claims and Appendix E suite
- Security and governance metadata
- Pipeline packages (`.dpcspkg`)
- JSON Schema / OpenAPI helpers
- Python and WASM bindings (PyPI `dpcs`, npm `@eddiethedean/dpcs`, Wasmer `eddiethedean/dpcs`)
- Report renderers (Markdown / HTML / Mermaid / DOT) and rich CLI formats
- Interactive TUI inspector
- Performance (parallel/incremental validate, AnalysisContext, Criterion benches)
- SPEC coverage matrix and diagnostic catalog
- CLI

See [`BINDINGS.md`](BINDINGS.md) for install and release channel names.
See [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md) for chapter-level completeness.

Future milestones:

- Stable 1.0.0
