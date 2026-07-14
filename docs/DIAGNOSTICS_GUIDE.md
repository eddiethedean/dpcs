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

## Categories (0.6.0)

| Constant | Wire value |
| --- | --- |
| `DOCUMENT` | `document` |
| `CANONICAL_OBJECT_MODEL` | `canonicalObjectModel` |
| `STRUCTURAL` | `structural` |
| `GRAPH` | `graph` |
| `REFERENCE` | `reference` |
| `DATA_FLOW` | `dataFlow` |
| `CONTROL_FLOW` | `controlFlow` |
| `EXECUTION_REQUIREMENTS` | `executionRequirements` |
| `SCHEDULING` | `scheduling` |
| `QUALITY_GATES` | `qualityGates` |
| `FAILURE_SEMANTICS` | `failureSemantics` |
| `LINEAGE` | `lineage` |
| `PLANNING` | `planning` |
| `EXTENSION` | `extension` |
| `SYNTAX` | `syntax` |

Planning failures use `DiagnosticStage::Planning` (for example `DPCS-PLN-001`).

Diagnostics describe observations only. They must not change pipeline semantics.
