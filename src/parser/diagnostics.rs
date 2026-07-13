//! Parse-stage diagnostics for invalid documents.

use crate::diagnostics::{Diagnostic, ValidationReport};
use crate::error::Error;

/// Convert a YAML parse/deserialize failure into an invalid-document error.
pub fn yaml_parse_error(err: serde_yaml::Error) -> Error {
    Error::InvalidDocument {
        report: report_from_yaml_error(&err),
    }
}

/// Convert a JSON parse/deserialize failure into an invalid-document error.
pub fn json_parse_error(err: serde_json::Error) -> Error {
    Error::InvalidDocument {
        report: report_from_json_error(&err),
    }
}

/// Attach a source document path to parse diagnostics when present.
pub fn with_source_path(mut err: Error, path: &std::path::Path) -> Error {
    let Error::InvalidDocument { report } = &mut err else {
        return err;
    };

    let path_display = path.display().to_string();
    for diagnostic in &mut report.diagnostics {
        diagnostic.source_location = Some(match diagnostic.source_location.take() {
            Some(location) => format!("{path_display}: {location}"),
            None => path_display.clone(),
        });
    }
    err
}

fn report_from_yaml_error(err: &serde_yaml::Error) -> ValidationReport {
    let message = err.to_string();
    let (id, remediation) = classify_message(&message);
    let mut diagnostic = Diagnostic::parse_error(id, message.clone()).with_remediation(remediation);

    if let Some(location) = err.location() {
        diagnostic = diagnostic.with_source_location(format!(
            "line {}, column {}",
            location.line(),
            location.column()
        ));
    } else if let Some(location) = location_from_message(&message) {
        diagnostic = diagnostic.with_source_location(location);
    }

    let mut report = ValidationReport::new();
    report.push(diagnostic);
    report.sort_deterministic();
    report
}

fn report_from_json_error(err: &serde_json::Error) -> ValidationReport {
    let message = err.to_string();
    let (id, remediation) = classify_message(&message);
    let mut diagnostic = Diagnostic::parse_error(id, message.clone()).with_remediation(remediation);

    if err.line() > 0 {
        diagnostic = diagnostic.with_source_location(format!(
            "line {}, column {}",
            err.line(),
            err.column()
        ));
    } else if let Some(location) = location_from_message(&message) {
        diagnostic = diagnostic.with_source_location(location);
    }

    let mut report = ValidationReport::new();
    report.push(diagnostic);
    report.sort_deterministic();
    report
}

fn classify_message(message: &str) -> (&'static str, String) {
    let lower = message.to_ascii_lowercase();
    if lower.contains("missing field") || lower.contains("missing struct field") {
        let remediation = match quoted_token(message) {
            Some(field) => format!("Provide the required field `{field}`"),
            None => {
                "Provide all required fields such as dpcsVersion, id, version, interface, and graph"
                    .to_owned()
            }
        };
        ("DPCS-PARSE-002", remediation)
    } else if lower.contains("invalid type") || lower.contains("invalid value") {
        (
            "DPCS-PARSE-002",
            "Ensure field values match the Pipeline Contract schema types".to_owned(),
        )
    } else {
        (
            "DPCS-PARSE-001",
            "Fix YAML or JSON syntax and ensure the document matches the Pipeline Contract schema"
                .to_owned(),
        )
    }
}

fn quoted_token(message: &str) -> Option<&str> {
    let start = message.find('`').or_else(|| message.find('"'))?;
    let quote = message.as_bytes()[start] as char;
    let rest = &message[start + 1..];
    let end = rest.find(quote)?;
    let token = &rest[..end];
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn location_from_message(message: &str) -> Option<String> {
    let lower = message.to_ascii_lowercase();
    let line_idx = lower.find("line ")?;
    let after_line = &message[line_idx + 5..];
    let line_digits_end = after_line
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(idx, _)| idx)
        .unwrap_or(after_line.len());
    if line_digits_end == 0 {
        return None;
    }
    let line = &after_line[..line_digits_end];

    let col_search = after_line[line_digits_end..].to_ascii_lowercase();
    let col_rel = col_search.find("column ")?;
    let after_col = &after_line[line_digits_end + col_rel + 7..];
    let col_digits_end = after_col
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(idx, _)| idx)
        .unwrap_or(after_col.len());
    if col_digits_end == 0 {
        return None;
    }
    let column = &after_col[..col_digits_end];
    Some(format!("line {line}, column {column}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_missing_field_with_named_remediation() {
        let (id, remediation) = classify_message("missing field `type` at line 12 column 3");
        assert_eq!(id, "DPCS-PARSE-002");
        assert!(remediation.contains("`type`"));
    }

    #[test]
    fn classifies_invalid_type_as_schema_not_syntax() {
        let (id, remediation) =
            classify_message("dpcsVersion: invalid type: integer, expected a string");
        assert_eq!(id, "DPCS-PARSE-002");
        assert!(remediation.contains("schema types"));
    }

    #[test]
    fn parses_location_embedded_in_message() {
        assert_eq!(
            location_from_message("missing field `id` at line 4 column 1").as_deref(),
            Some("line 4, column 1")
        );
    }
}
