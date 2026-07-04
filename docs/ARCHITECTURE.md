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

Initial implementation scope:

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
```

The first Rust crate should stop at diagnostics and add only a lightweight Pipeline Plan skeleton.
