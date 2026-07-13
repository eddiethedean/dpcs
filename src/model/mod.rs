//! Canonical Object Model (COM) for DPCS Pipeline Contracts.
//!
//! Types in this module are serialization-independent representations of the
//! objects defined in `SPEC.md`. Serde attributes exist only at the boundary.

mod compatibility;
mod contract;
mod control_flow;
mod data_flow;
mod execution;
mod extension;
mod extension_value;
mod failure;
mod graph;
mod identity;
mod interface;
mod invariants;
mod lineage;
mod metadata;
mod quality_gate;
mod reference;
mod registry;
mod scheduling;
mod step;
mod versioning;

pub use compatibility::*;
pub use contract::*;
pub use control_flow::*;
pub use data_flow::*;
pub use execution::*;
pub use extension::*;
pub use extension_value::{ExtensionMap, ExtensionValue};
pub use failure::*;
pub use graph::*;
pub use identity::*;
pub use interface::*;
pub use invariants::validate_com_invariants;
pub use lineage::*;
pub use metadata::*;
pub use quality_gate::*;
pub use reference::*;
pub use registry::*;
pub use scheduling::*;
pub use step::*;
pub use versioning::*;
