//! Pipeline interface model.
//!
//! Every Pipeline Contract defines exactly one [`PipelineInterface`] that
//! represents the complete external boundary (SPEC Ch 4 §3).
//!
//! Each [`InterfacePort`] SHALL possess a stable identifier, interface name,
//! declared contract reference, and logical purpose (SPEC Ch 4 §4–5). Missing
//! properties are reported by COM invariant validation.

use serde::{Deserialize, Serialize};

use super::{ExtensionMap, Metadata};

/// External boundary of a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PipelineInterface {
    /// Pipeline inputs.
    #[serde(default)]
    pub inputs: Vec<InterfacePort>,
    /// Pipeline outputs.
    #[serde(default)]
    pub outputs: Vec<InterfacePort>,
    /// Interface metadata required by this specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl PipelineInterface {
    /// Returns the input port with the given identifier.
    pub fn input(&self, id: &str) -> Option<&InterfacePort> {
        self.inputs.iter().find(|port| port.id == id)
    }

    /// Returns the output port with the given identifier.
    pub fn output(&self, id: &str) -> Option<&InterfacePort> {
        self.outputs.iter().find(|port| port.id == id)
    }

    /// Returns all input and output ports.
    pub fn all_ports(&self) -> impl Iterator<Item = &InterfacePort> {
        self.inputs.iter().chain(self.outputs.iter())
    }

    /// Returns `true` when all port identifiers are unique within the interface.
    pub fn has_unique_port_ids(&self) -> bool {
        let mut seen = std::collections::BTreeSet::new();
        self.all_ports().all(|port| seen.insert(port.id.as_str()))
    }
}

/// A single input or output port on the pipeline interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InterfacePort {
    /// Stable port identifier.
    pub id: String,
    /// Interface name for this port (SPEC Ch 4 §4–5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Reference to an external contract (for example ODCS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Logical purpose of the port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl InterfacePort {
    /// Returns `true` when all SPEC-required port properties are present.
    pub fn is_complete(&self) -> bool {
        !self.id.trim().is_empty()
            && self
                .name
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .contract_ref
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .purpose
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
    }
}
