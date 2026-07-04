# Non-Goals for First Implementation

Do not implement yet:

- Airflow generation
- Dagster generation
- Prefect generation
- Temporal generation
- Kubernetes manifests
- execution runtime
- scheduling runtime
- actual data movement
- ETL execution
- ODCS validation internals
- DTCS validation internals

DPCS should reference ODCS and DTCS contracts, not implement them internally in the first release.
