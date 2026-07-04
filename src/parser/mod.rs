//! YAML and JSON parsers for DPCS documents.

mod json;
mod yaml;

pub use json::{parse_json, parse_json_file};
pub use yaml::{parse_yaml, parse_yaml_file};

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
