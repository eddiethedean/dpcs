# Cursor Build Prompt

Build the Rust reference implementation of DPCS.

The repository contains a single `SPEC.md` document with the full 26-chapter DPCS draft specification.

Treat `SPEC.md` as authoritative.

Processing pipeline through ROADMAP 0.10.0:

```text
DPCS Document -> Parser -> COM -> Validator -> Pipeline Plan -> Capability Evaluation -> Orchestrator Binding
(+ Compatibility / Registry documents+HTTP / Conformance / Packages / Schema emit)
```

Implemented:

1. Rust workspace (`dpcs` lib, `dpcs-cli`)
2. Canonical Object Model
3. YAML and JSON parsing
4. Diagnostics model (including DiagnosticReport)
5. Phase-based validation (including extensions, security, governance)
6. Pipeline graph validation
7. Data Flow / Control Flow validation
8. Pipeline Plan
9. Capability profiles and matching
10. Orchestrator binding scaffolds (Airflow, Dagster, Prefect, Temporal, Kubernetes)
11. Compatibility analysis
12. Registry document validation + reference HTTP client/server (ADR-0005)
13. Conformance claims and Appendix E suite
14. Pipeline packages (`.dpcspkg`)
15. JSON Schema / OpenAPI helpers
16. Python and WASM bindings
17. CLI
18. Tests and fixtures

Do not implement execution runtimes or production-grade operator libraries.

Use Rust best practices and keep names aligned with `SPEC.md`.
