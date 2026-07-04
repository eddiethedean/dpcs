# Diagnostics Guide

Suggested types:

```rust
pub enum Severity {
    Error,
    Warning,
    Information,
}

pub enum DiagnosticStage {
    Parse,
    CanonicalObjectModel,
    Validation,
    Planning,
    CapabilityEvaluation,
    OrchestratorBinding,
}

pub struct Diagnostic {
    pub id: String,
    pub severity: Severity,
    pub stage: DiagnosticStage,
    pub category: String,
    pub message: String,
    pub object_ref: Option<String>,
    pub remediation: Option<String>,
}
```

Diagnostics describe observations only. They must not change pipeline semantics.
