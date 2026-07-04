//! YAML parsing.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::model::PipelineContract;

/// Parse a Pipeline Contract from a YAML string.
pub fn parse_yaml(input: &str) -> Result<PipelineContract> {
    Ok(serde_yaml::from_str(input)?)
}

/// Parse a Pipeline Contract from a YAML file.
pub fn parse_yaml_file(path: impl AsRef<Path>) -> Result<PipelineContract> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_yaml(&contents)
}
