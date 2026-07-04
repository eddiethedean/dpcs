# Cursor Build Prompt

Build the first Rust implementation of DPCS.

The repository contains a single `SPEC.md` document with the full 26-chapter DPCS draft specification.

Treat `SPEC.md` as authoritative.

Initial goal:

```text
DPCS Document -> Parser -> Canonical Object Model -> Validator -> Diagnostics
```

Implement:

1. Rust crate skeleton
2. Canonical Object Model
3. YAML and JSON parsing
4. Diagnostics model
5. Phase-based validation
6. Pipeline graph validation
7. Data Flow validation
8. Control Flow validation
9. CLI
10. Tests and fixtures

Do not implement orchestrator binding, execution runtime, or Airflow/Dagster/Prefect generation yet.

Use Rust best practices and keep names aligned with `SPEC.md`.
