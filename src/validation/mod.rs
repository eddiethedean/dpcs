//! Phase-based validation for Pipeline Contracts.

mod com;
mod control_flow;
mod data_flow;
mod document;
mod execution;
mod extensions;
mod failure;
mod graph;
mod lineage;
mod phases;
mod quality;
mod references;
mod scheduling;
mod structural;

pub use phases::validate;
