# Non-Goals Beyond 0.10.0

Completed through 0.10.0: orchestrator binding scaffolds, SPEC Ch 18–25 document
models, JSON Schema/OpenAPI helpers, reference registry HTTP API, pipeline
packages, and Python/WASM distribution packages (PyPI `dpcs`, npm
`@eddiethedean/dpcs`, Wasmer `eddiethedean/dpcs`; see [`BINDINGS.md`](BINDINGS.md)).

Do **not** implement yet:

- production-complete Airflow operators / Dagster resources / Temporal workers
- execution runtime
- scheduling runtime
- actual data movement
- ETL execution
- ODCS validation internals
- DTCS validation internals
- TUI / rich HTML reports (ROADMAP 0.11)

DPCS should reference ODCS and DTCS contracts, not implement them internally.
Binding produces orchestration artifacts; it does not run pipelines.
The reference registry HTTP API is toolkit-local (ADR-0005), not the sole future SPEC protocol.
