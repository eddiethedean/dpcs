//! Phase-based validation for Pipeline Contracts.

mod com;
mod control_flow;
mod data_flow;
mod document;
mod extensions;
mod failure;
mod graph;
mod phases;
mod quality;
mod references;
mod structural;

pub use phases::validate;
