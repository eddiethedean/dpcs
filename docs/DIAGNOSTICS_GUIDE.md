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
    CompatibilityAnalysis,
    Planning,
    CapabilityEvaluation,
    OrchestratorBinding,
    ExecutionAnalysis, // reserved
}

pub struct Diagnostic {
    pub id: String,
    pub severity: Severity,
    pub stage: DiagnosticStage,
    pub category: String,
    pub message: String,
    pub object_ref: Option<String>,
    pub remediation: Option<String>,
    pub source_location: Option<String>,
    pub related_diagnostics: Vec<String>,
    pub metadata: Option<BTreeMap<String, String>>,
}
```

`ValidationReport` aggregates diagnostics. `DiagnosticReport` adds
`processingResult`, optional `artifactId`, and `implementation` metadata.

Use `validate_diagnostic` for structural self-checks (`DPCS-DIAG-001`–`004`).

## Categories (0.9.0)

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
| `CAPABILITY` | `capability` |
| `BINDING` | `binding` |
| `COMPATIBILITY` | `compatibility` |
| `VERSIONING` | `versioning` |
| `REGISTRY` | `registry` |
| `CONFORMANCE` | `conformance` |
| `SECURITY` | `security` |
| `GOVERNANCE` | `governance` |
| `EXTENSION` | `extension` |
| `SYNTAX` | `syntax` |

Planning failures use `DiagnosticStage::Planning` (for example `DPCS-PLN-001`).
Capability match failures use `DiagnosticStage::CapabilityEvaluation`
(`DPCS-CAP-001`–`006`). Binding failures use `DiagnosticStage::OrchestratorBinding`:

| ID | Meaning |
| --- | --- |
| `DPCS-BIND-001` | Capability gate failed before translation |
| `DPCS-BIND-002` | Unknown binding target |
| `DPCS-BIND-003` | Translation incomplete / empty artifacts |
| `DPCS-BIND-004` | Artifact write failure or unsafe relative path |

Compatibility analysis uses `DiagnosticStage::CompatibilityAnalysis` (`DPCS-COMPAT-*`).
Unknown preserved extensions emit `Information` (`DPCS-EXT-010`).

Diagnostics describe observations only. They must not change pipeline semantics.
