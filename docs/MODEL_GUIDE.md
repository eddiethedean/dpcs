# Canonical Object Model Guide

The Canonical Object Model (COM) is the serialization-independent representation
of a DPCS Pipeline Contract (SPEC Ch 3).

## Root type

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
    pub compatibility: Option<CompatibilityPolicy>,
    pub extensions: ExtensionMap,
}
```

`SPEC.md` is authoritative. When the specification and this guide differ, follow
the specification.

## Serialization independence

COM types are the canonical in-memory representation. Serde attributes exist at
the wire boundary so YAML and JSON documents deserialize into the same COM values.

Extension fields use [`ExtensionValue`](../../src/model/extension_value.rs) and
[`ExtensionMap`](../../src/model/extension_value.rs), not `serde_json::Value`.
Conversions to and from JSON-shaped values happen only at parse/serialize time.

## Identity model

Every addressable COM object possesses a stable identifier within the contract
(SPEC Ch 3 §5).

| Type | Purpose |
| --- | --- |
| `ObjectId` | Stable identifier newtype (`is_present()` reports emptiness) |
| `PipelineIdentity` | Root pipeline identity (`id`, `version`, `dpcsVersion`, `name`) |
| `ObjectKind` | Kind of addressable object (step, interface port, …) |
| `ObjectPath` | Deterministic path (`interface.inputs.<id>`, `steps.<id>`, …) |
| `IdentityCatalog` | Catalog of all addressable objects in a contract |

```rust
let contract = PipelineContract::from_yaml_file("pipeline.dpcs.yaml")?;
let identity = contract.identity();
let catalog = contract.identity_catalog();
```

## Pipeline interface

Every contract defines exactly one [`PipelineInterface`] (SPEC Ch 4 §3).

Each [`InterfacePort`] SHALL possess:

- a stable identifier (`id`)
- an interface name (`name`)
- a declared contract reference (`contractRef`)
- a logical purpose (`purpose`)

Ports deserialize with optional fields for ergonomics. COM invariant validation
reports missing properties using `canonicalObjectModel` diagnostics.

```rust
contract.interface.input("customer_raw");
contract.interface.output("customer_clean");
contract.interface.all_ports();
```

## Metadata

DPCS names metadata as a root and interface slot. This crate provides an initial
profile on [`Metadata`]:

- `description`
- `owner`
- `tags`
- extension fields

Additional metadata MAY be supplied through extension fields.

## Pipeline graph (0.4.0)

[`PipelineGraph`] includes entry/exit points, optional metadata, and directed edges:

```rust
pub struct PipelineGraph {
    pub entry_points: Vec<String>,
    pub exit_points: Vec<String>,
    pub metadata: Option<Metadata>,
    pub edges: Vec<GraphEdge>,
    pub extensions: ExtensionMap,
}
```

[`DataFlow`] may declare an associated `contractRef`. [`DependencyGraph`] builds a
directed step dependency graph from explicit graph edges, control flow, and
data flow where both endpoints resolve to steps.

```rust
use dpcs::DependencyGraph;

let graph = DependencyGraph::from_contract(&contract);
let order = graph.topological_order()?;
```

## COM validation

COM invariants run as the first validation phase after document checks:

1. Pipeline identity completeness
2. Addressable object identifier presence and uniqueness
3. Interface port completeness
4. Extension key collision with reserved root fields

Later validation phases (structural, graph, references, flows) build on the COM.

Full validation polish for data/control flow completeness shipped in roadmap
0.5.0: dataset identity, port satisfiability, endpoint roles, conflicting deps,
and richer reference checks. Quality/failure/execution rule evaluation remains
roadmap 0.6.0.

[`PipelineGraph`]: ../../src/model/graph.rs
[`DataFlow`]: ../../src/model/data_flow.rs
[`DependencyGraph`]: ../../src/model/analysis.rs

[`PipelineInterface`]: ../../src/model/interface.rs
[`InterfacePort`]: ../../src/model/interface.rs
[`Metadata`]: ../../src/model/metadata.rs
