//! JSON parsing and serialization.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::model::PipelineContract;

use super::diagnostics;

/// Parse a Pipeline Contract from a JSON string.
pub fn parse_json(input: &str) -> Result<PipelineContract> {
    serde_json::from_str(input).map_err(diagnostics::json_parse_error)
}

/// Parse a Pipeline Contract from a JSON file.
pub fn parse_json_file(path: impl AsRef<Path>) -> Result<PipelineContract> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_json(&contents).map_err(|err| diagnostics::with_source_path(err, path))
}

/// Serialize a Pipeline Contract to a pretty-printed JSON string.
///
/// Reserved root field names that collide in `extensions` are omitted so the
/// wire document does not emit duplicate keys.
pub fn to_json(contract: &PipelineContract) -> Result<String> {
    let wire = contract.for_wire_serialization();
    serde_json::to_string_pretty(&wire)
        .map_err(|err| Error::Serialization(format!("failed to serialize JSON: {err}")))
}

/// Serialize a Pipeline Contract to a JSON file.
pub fn to_json_file(contract: &PipelineContract, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let contents = to_json(contract)?;
    fs::write(path, contents).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}
