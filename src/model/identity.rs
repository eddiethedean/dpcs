//! Identity model for addressable Canonical Object Model objects.
//!
//! Every addressable COM object possesses a stable identifier within the scope
//! of a Pipeline Contract (SPEC Ch 3 §5).

use std::collections::BTreeMap;
use std::fmt;

use super::PipelineContract;

/// A stable object identifier within a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(String);

impl ObjectId {
    /// Create an object identifier from a raw string.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Returns the raw identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` when the identifier is non-empty after trimming.
    pub fn is_present(&self) -> bool {
        !self.0.trim().is_empty()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ObjectId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ObjectId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Pipeline-level identity extracted from the root contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineIdentity {
    /// Stable pipeline identifier.
    pub id: ObjectId,
    /// Pipeline contract version.
    pub version: ObjectId,
    /// DPCS specification version targeted by the contract.
    pub dpcs_version: ObjectId,
    /// Optional human-readable pipeline name.
    pub name: Option<String>,
}

impl PipelineIdentity {
    /// Returns `true` when required identity fields are present.
    pub fn is_complete(&self) -> bool {
        self.id.is_present() && self.version.is_present() && self.dpcs_version.is_present()
    }
}

/// Kind of addressable object within a Pipeline Contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObjectKind {
    /// Root pipeline contract.
    Pipeline,
    /// Interface input port.
    InterfaceInput,
    /// Interface output port.
    InterfaceOutput,
    /// Pipeline step.
    Step,
    /// External contract reference.
    ContractReference,
    /// Quality gate.
    QualityGate,
    /// Failure semantics declaration.
    FailureSemantics,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pipeline => write!(f, "pipeline"),
            Self::InterfaceInput => write!(f, "interfaceInput"),
            Self::InterfaceOutput => write!(f, "interfaceOutput"),
            Self::Step => write!(f, "step"),
            Self::ContractReference => write!(f, "contractReference"),
            Self::QualityGate => write!(f, "qualityGate"),
            Self::FailureSemantics => write!(f, "failureSemantics"),
        }
    }
}

/// Deterministic path to an addressable object within a contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectPath(String);

impl ObjectPath {
    /// Create an object path from a raw path string.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Returns the path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Path to the root pipeline object.
    pub fn pipeline() -> Self {
        Self("pipeline".to_owned())
    }

    /// Path to an interface input port.
    pub fn interface_input(id: &str) -> Self {
        Self(format!("interface.inputs.{id}"))
    }

    /// Path to an interface output port.
    pub fn interface_output(id: &str) -> Self {
        Self(format!("interface.outputs.{id}"))
    }

    /// Path to a pipeline step.
    pub fn step(id: &str) -> Self {
        Self(format!("steps.{id}"))
    }

    /// Path to a contract reference.
    pub fn contract_reference(id: &str) -> Self {
        Self(format!("contractReferences.{id}"))
    }

    /// Path to a quality gate.
    pub fn quality_gate(id: &str) -> Self {
        Self(format!("qualityGates.{id}"))
    }

    /// Path to a failure semantics declaration.
    pub fn failure_semantics(id: &str) -> Self {
        Self(format!("failureSemantics.{id}"))
    }
}

impl fmt::Display for ObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Catalog entry for an addressable COM object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityEntry {
    /// Object kind.
    pub kind: ObjectKind,
    /// Stable object identifier.
    pub id: ObjectId,
    /// Deterministic object path.
    pub path: ObjectPath,
}

/// Catalog of addressable objects within a Pipeline Contract.
#[derive(Debug, Clone, Default)]
pub struct IdentityCatalog {
    entries: Vec<IdentityEntry>,
    by_path: BTreeMap<String, usize>,
    by_kind_and_id: BTreeMap<(ObjectKind, String), usize>,
}

impl IdentityCatalog {
    /// Build an identity catalog from a Pipeline Contract.
    pub fn from_contract(contract: &PipelineContract) -> Self {
        let mut catalog = Self::default();

        catalog.insert(
            ObjectKind::Pipeline,
            ObjectId::new(contract.id.clone()),
            ObjectPath::pipeline(),
        );

        for port in &contract.interface.inputs {
            catalog.insert(
                ObjectKind::InterfaceInput,
                ObjectId::new(port.id.clone()),
                ObjectPath::interface_input(&port.id),
            );
        }

        for port in &contract.interface.outputs {
            catalog.insert(
                ObjectKind::InterfaceOutput,
                ObjectId::new(port.id.clone()),
                ObjectPath::interface_output(&port.id),
            );
        }

        for step in &contract.steps {
            catalog.insert(
                ObjectKind::Step,
                ObjectId::new(step.id.clone()),
                ObjectPath::step(&step.id),
            );
        }

        for reference in &contract.contract_references {
            catalog.insert(
                ObjectKind::ContractReference,
                ObjectId::new(reference.id.clone()),
                ObjectPath::contract_reference(&reference.id),
            );
        }

        for gate in &contract.quality_gates {
            catalog.insert(
                ObjectKind::QualityGate,
                ObjectId::new(gate.id.clone()),
                ObjectPath::quality_gate(&gate.id),
            );
        }

        for failure in &contract.failure_semantics {
            catalog.insert(
                ObjectKind::FailureSemantics,
                ObjectId::new(failure.id.clone()),
                ObjectPath::failure_semantics(&failure.id),
            );
        }

        catalog
    }

    fn insert(&mut self, kind: ObjectKind, id: ObjectId, path: ObjectPath) {
        let index = self.entries.len();
        self.by_path.insert(path.as_str().to_owned(), index);
        self.by_kind_and_id
            .insert((kind, id.as_str().to_owned()), index);
        self.entries.push(IdentityEntry { kind, id, path });
    }

    /// Returns all catalog entries in deterministic order.
    pub fn entries(&self) -> &[IdentityEntry] {
        &self.entries
    }

    /// Look up an entry by object path.
    pub fn get_by_path(&self, path: &str) -> Option<&IdentityEntry> {
        self.by_path.get(path).map(|index| &self.entries[*index])
    }

    /// Look up an entry by kind and identifier.
    pub fn get_by_kind_and_id(&self, kind: ObjectKind, id: &str) -> Option<&IdentityEntry> {
        self.by_kind_and_id
            .get(&(kind, id.to_owned()))
            .map(|index| &self.entries[*index])
    }

    /// Returns duplicate identifier groups within a kind.
    pub fn duplicate_ids_by_kind(&self) -> BTreeMap<ObjectKind, Vec<ObjectId>> {
        let mut counts: BTreeMap<(ObjectKind, String), usize> = BTreeMap::new();

        for entry in &self.entries {
            if entry.kind == ObjectKind::Pipeline {
                continue;
            }
            *counts
                .entry((entry.kind, entry.id.as_str().to_owned()))
                .or_default() += 1;
        }

        let mut duplicates = BTreeMap::new();
        for ((kind, id), count) in counts {
            if count > 1 {
                duplicates
                    .entry(kind)
                    .or_insert_with(Vec::new)
                    .push(ObjectId::new(id));
            }
        }

        duplicates
    }

    /// Returns entries whose identifiers are empty or whitespace-only.
    pub fn entries_with_missing_ids(&self) -> Vec<&IdentityEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind != ObjectKind::Pipeline && !entry.id.is_present())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn builds_catalog_from_contract() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
version: "0.1.0"
interface:
  inputs:
    - id: "in"
      name: "In"
      contractRef: "ref"
      purpose: "input"
  outputs: []
steps:
  - id: "step_a"
    type: "dtcs:transform"
graph:
  edges: []
"#,
        )
        .unwrap();

        let catalog = IdentityCatalog::from_contract(&contract);
        assert!(catalog.get_by_path("pipeline").is_some());
        assert!(catalog
            .get_by_kind_and_id(ObjectKind::Step, "step_a")
            .is_some());
        assert!(catalog.get_by_path("interface.inputs.in").is_some());
    }
}
