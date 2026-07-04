# Canonical Object Model Guide

Initial root type sketch:

```rust
pub struct PipelineContract {
    pub dpcs_version: String,
    pub id: String,
    pub name: Option<String>,
    pub version: String,
    pub metadata: Option<Metadata>,
    pub interface: PipelineInterface,
    pub graph: PipelineGraph,
    pub steps: Vec<PipelineStep>,
    pub contract_references: Vec<ContractReference>,
    pub data_flow: Vec<DataFlow>,
    pub control_flow: Vec<ControlFlow>,
    pub execution: Option<ExecutionRequirements>,
    pub scheduling: Option<SchedulingIntent>,
    pub quality_gates: Vec<QualityGate>,
    pub failure_semantics: Vec<FailureSemantics>,
    pub lineage: Option<PipelineLineage>,
    pub extensions: IndexMap<String, serde_json::Value>,
}
```

This is a starting sketch. `SPEC.md` is authoritative.
