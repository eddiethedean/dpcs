//! YAML and JSON parsers for DPCS documents.

mod diagnostics;
mod json;
mod yaml;

pub use json::{parse_json, parse_json_file, parse_json_slice, to_json, to_json_file};
pub use yaml::{parse_yaml, parse_yaml_file, parse_yaml_slice, to_yaml, to_yaml_file};

use std::path::Path;

use crate::error::{Error, Result};
use crate::model::PipelineContract;

/// Parse a Pipeline Contract from a file, dispatching on extension.
pub fn parse_file(path: impl AsRef<Path>) -> Result<PipelineContract> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("yaml") | Some("yml") => parse_yaml_file(path),
        Some("json") => parse_json_file(path),
        _ => Err(Error::UnsupportedFormat {
            path: path.to_path_buf(),
        }),
    }
}

/// Serialize a Pipeline Contract to a file, dispatching on extension.
pub fn to_file(contract: &PipelineContract, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("yaml") | Some("yml") => to_yaml_file(contract, path),
        Some("json") => to_json_file(contract, path),
        _ => Err(Error::UnsupportedFormat {
            path: path.to_path_buf(),
        }),
    }
}
