# Non-Goals for Binding and Beyond

Orchestrator binding scaffold generation is implemented in 0.8.0. Do **not**
implement yet:

- production-complete Airflow operators / Dagster resources / Temporal workers
- execution runtime
- scheduling runtime
- actual data movement
- ETL execution
- ODCS validation internals
- DTCS validation internals
- Binding Profiles registry clients (ROADMAP 0.9+)

DPCS should reference ODCS and DTCS contracts, not implement them internally.
Binding produces orchestration artifacts; it does not run pipelines.
