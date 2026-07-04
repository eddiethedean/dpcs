//! JSON parsing.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::model::PipelineContract;

/// Parse a Pipeline Contract from a JSON string.
pub fn parse_json(input: &str) -> Result<PipelineContract> {
    Ok(serde_json::from_str(input)?)
}

/// Parse a Pipeline Contract from a JSON file.
pub fn parse_json_file(path: impl AsRef<Path>) -> Result<PipelineContract> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_json(&contents)
}
