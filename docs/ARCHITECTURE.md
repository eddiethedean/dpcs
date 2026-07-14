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

Current implementation scope (through ROADMAP 0.6.0):

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
```

Capability evaluation and orchestrator binding remain roadmap 0.7.0–0.8.0.
Execution runtimes are out of scope.
