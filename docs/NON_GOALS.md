# Non-Goals Beyond 0.9.0

Completed through 0.9.0: orchestrator binding scaffolds and SPEC Ch 18–25
document models (including in-process registry validation).

Do **not** implement yet:

- production-complete Airflow operators / Dagster resources / Temporal workers
- execution runtime
- scheduling runtime
- actual data movement
- ETL execution
- ODCS validation internals
- DTCS validation internals
- Network registry clients / Binding Profiles registry clients (ROADMAP 0.10)
- Python / WASM bindings, JSON Schema generation (ROADMAP 0.10)

DPCS should reference ODCS and DTCS contracts, not implement them internally.
Binding produces orchestration artifacts; it does not run pipelines.
Registry document validation does not imply a remote registry protocol.
