//! YAML parsing and serialization.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::model::PipelineContract;

use super::diagnostics;

/// Parse a Pipeline Contract from a YAML string.
pub fn parse_yaml(input: &str) -> Result<PipelineContract> {
    serde_yaml::from_str(input).map_err(diagnostics::yaml_parse_error)
}

/// Parse a Pipeline Contract from YAML bytes (UTF-8).
pub fn parse_yaml_slice(input: &[u8]) -> Result<PipelineContract> {
    let input = std::str::from_utf8(input)
        .map_err(|err| Error::Serialization(format!("YAML input is not valid UTF-8: {err}")))?;
    parse_yaml(input)
}

/// Parse a Pipeline Contract from a YAML file.
pub fn parse_yaml_file(path: impl AsRef<Path>) -> Result<PipelineContract> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_yaml(&contents).map_err(|err| diagnostics::with_source_path(err, path))
}

/// Serialize a Pipeline Contract to a YAML string.
///
/// Reserved root field names that collide in `extensions` are omitted so the
/// wire document does not emit duplicate keys (without cloning the COM tree).
pub fn to_yaml(contract: &PipelineContract) -> Result<String> {
    serde_yaml::to_string(contract)
        .map_err(|err| Error::Serialization(format!("failed to serialize YAML: {err}")))
}

/// Serialize a Pipeline Contract to a YAML file.
pub fn to_yaml_file(contract: &PipelineContract, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let contents = to_yaml(contract)?;
    fs::write(path, contents).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}
