# Architecture

DPCS follows the canonical processing architecture:

```text
Pipeline Contract
        │
        ▼
Canonical Object Model
        │
        ▼
Validation
        │
        ▼
Pipeline Plan
        │
        ▼
Capability Evaluation
        │
        ▼
Orchestrator Binding
        │
        ▼
Execution Runtime
```

Current implementation scope (through ROADMAP 0.8.0):

```text
DPCS Document
        │
        ▼
Parser
        │
        ▼
Canonical Object Model
        │
        ▼
Validator
        │
        ▼
Diagnostics
        │
        ▼
Pipeline Plan (deterministic, validation-gated)
        │
        ▼
Capability Evaluation (profile match, no plan mutation)
        │
        ▼
Orchestrator Binding (scaffold artifacts; capability-gated)
```

Execution runtimes remain out of scope. Binding adapters emit implementation-defined
scaffold artifacts for Airflow, Dagster, Prefect, Temporal, and Kubernetes.
