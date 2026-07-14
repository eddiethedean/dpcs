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

Current implementation scope (through ROADMAP 0.12.0):

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
Validator (+ extensions, security, governance, versioning)
        │
        ▼
Diagnostics / DiagnosticReport
        │
        ▼
Pipeline Plan (deterministic, validation-gated)
        │
        ▼
Capability Evaluation (profile match, no plan mutation)
        │
        ▼
Orchestrator Binding (scaffold artifacts; capability-gated)

Also first-class (not mutating contracts):
  Compatibility analysis · Registry document validate · Registry HTTP client/server
  Conformance claims/profiles · Pipeline packages · JSON Schema / OpenAPI emit
  Language bindings (Python / WASM; see BINDINGS.md)
  Reports (Markdown/HTML/Mermaid/DOT) · Rich CLI formats · TUI inspector
```

Execution runtimes remain out of scope. Binding adapters emit implementation-defined
scaffold artifacts for Airflow, Dagster, Prefect, Temporal, and Kubernetes.
Registry networking uses the reference HTTP API (see ADR-0005).
Language packages publish to PyPI (`dpcs`), npm (`@eddiethedean/dpcs`), and
Wasmer (`eddiethedean/dpcs`).
