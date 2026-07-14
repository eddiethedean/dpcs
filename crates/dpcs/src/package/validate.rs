//! Package validation.

use std::collections::BTreeSet;
use std::path::Path;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::is_valid_version;
use crate::package::{PackageLayout, PackageManifest};
use crate::paths::{is_safe_relative, join_under_root};

/// Validate a package directory.
pub fn validate_package(root: impl AsRef<Path>) -> ValidationReport {
    let mut report = ValidationReport::new();
    let layout = match PackageLayout::open(root) {
        Ok(layout) => layout,
        Err(err) => {
            report.push(
                Diagnostic::error(
                    "DPCS-PKG-001",
                    categories::PACKAGE,
                    format!("failed to open package: {err}"),
                )
                .with_remediation("Ensure manifest.yaml exists and is well-formed"),
            );
            return report;
        }
    };
    validate_manifest(&layout.manifest, &mut report);
    let mut seen = BTreeSet::new();
    for (idx, entry) in layout.manifest.artifacts.iter().enumerate() {
        let key = format!("{}@{}", entry.id, entry.version);
        if !seen.insert(key) {
            report.push(
                Diagnostic::error(
                    "DPCS-PKG-002",
                    categories::PACKAGE,
                    format!(
                        "duplicate artifact id/version `{}@{}`",
                        entry.id, entry.version
                    ),
                )
                .with_object_ref(format!("artifacts[{idx}]")),
            );
        }
        if entry.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-PKG-003",
                    categories::PACKAGE,
                    "artifact id must be non-empty",
                )
                .with_object_ref(format!("artifacts[{idx}].id")),
            );
        }
        if !is_valid_version(&entry.version) {
            report.push(
                Diagnostic::error(
                    "DPCS-PKG-004",
                    categories::PACKAGE,
                    format!("invalid artifact version `{}`", entry.version),
                )
                .with_object_ref(format!("artifacts[{idx}].version")),
            );
        }
        if !is_safe_relative(&entry.path) {
            report.push(
                Diagnostic::error(
                    "DPCS-PKG-005",
                    categories::PACKAGE,
                    format!("unsafe or absolute artifact path `{}`", entry.path),
                )
                .with_object_ref(format!("artifacts[{idx}].path"))
                .with_remediation("Use a relative path without '..'"),
            );
            continue;
        }
        let path = match join_under_root(&layout.root, &entry.path) {
            Ok(path) => path,
            Err(err) => {
                report.push(
                    Diagnostic::error(
                        "DPCS-PKG-005",
                        categories::PACKAGE,
                        format!("artifact path escapes package root `{}`: {err}", entry.path),
                    )
                    .with_object_ref(format!("artifacts[{idx}].path")),
                );
                continue;
            }
        };
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                report.push(
                    Diagnostic::error(
                        "DPCS-PKG-010",
                        categories::PACKAGE,
                        format!("artifact path must not be a symlink: {}", entry.path),
                    )
                    .with_object_ref(format!("artifacts[{idx}].path")),
                );
            }
            Ok(meta) if meta.is_file() => {}
            Ok(_) | Err(_) => {
                report.push(
                    Diagnostic::error(
                        "DPCS-PKG-006",
                        categories::PACKAGE,
                        format!("artifact file missing: {}", entry.path),
                    )
                    .with_object_ref(format!("artifacts[{idx}].path")),
                );
            }
        }
    }
    report
}

fn validate_manifest(manifest: &PackageManifest, report: &mut ValidationReport) {
    if manifest.id.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-PKG-007",
                categories::PACKAGE,
                "package id must be non-empty",
            )
            .with_object_ref("id"),
        );
    }
    if !is_valid_version(&manifest.version) {
        report.push(
            Diagnostic::error(
                "DPCS-PKG-008",
                categories::PACKAGE,
                format!("invalid package version `{}`", manifest.version),
            )
            .with_object_ref("version"),
        );
    }
    if manifest.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-PKG-009",
                categories::PACKAGE,
                "package dpcsVersion must be non-empty",
            )
            .with_object_ref("dpcsVersion"),
        );
    }
}
