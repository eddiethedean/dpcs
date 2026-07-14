# Cursor Build Prompt

Build the Rust reference implementation of DPCS.

The repository contains a single `SPEC.md` document with the full 26-chapter DPCS draft specification.

Treat `SPEC.md` as authoritative.

Processing pipeline through ROADMAP 0.13.0:

```text
DPCS Document -> Parser -> COM -> Validator (+ resolve) -> Plan -> Capabilities -> Binding
(+ Compatibility / Registry documents+HTTP / Conformance / Packages / Schema emit / Reports / TUI)
(+ AnalysisContext / parallel+incremental validate / Criterion benches)
(+ SPEC_COVERAGE.md / diagnostics.catalog.json)
```

Implemented:

1. Rust workspace (`dpcs` lib, `dpcs-cli`)
2. Canonical Object Model
3. YAML and JSON parsing
4. Diagnostics model (including DiagnosticReport + catalog)
5. Phase-based validation (including extensions, security, governance)
6. Pipeline graph validation
7. Data Flow / Control Flow validation
8. Contract reference resolution and nested pipelines
9. Pipeline Plan (with nested provenance)
10. Capability profiles and matching
11. Orchestrator binding scaffolds + `dpcs_semantics.json` (Airflow, Dagster, Prefect, Temporal, Kubernetes)
12. Compatibility analysis
13. Registry document validation + reference HTTP client/server (ADR-0005)
14. Conformance claims and Appendix E suite
15. Pipeline packages (`.dpcspkg`)
16. JSON Schema / OpenAPI helpers
17. Python and WASM bindings (PyPI / npm `@eddiethedean/dpcs` / Wasmer)
18. Report module (Markdown / HTML / Mermaid / DOT) and rich CLI `--format`/`--out`
19. Interactive TUI inspector (`tui` feature)
20. Performance: `AnalysisContext`, `parallel` validate, `ValidationCache`, synth + Criterion
21. CLI
22. Tests and fixtures
23. SPEC coverage matrix and stable public API documentation

Do not implement execution runtimes or production-grade operator libraries.

Use Rust best practices and keep names aligned with `SPEC.md`.
