//! COM invariant checks for the Canonical Object Model.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, DiagnosticStage, Severity, ValidationReport};
use crate::model::{is_present_version, is_reserved_root_field, IdentityCatalog, PipelineContract};

/// Validate Canonical Object Model invariants for a Pipeline Contract.
pub fn validate_com_invariants(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    validate_pipeline_identity(contract, &mut report);
    validate_identity_catalog(contract, &mut report);
    validate_interface_port_uniqueness(contract, &mut report);
    validate_interface_ports(contract, &mut report);
    validate_extension_keys(contract, &mut report);

    report
}

fn com_error(id: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        id: id.to_owned(),
        severity: Severity::Error,
        stage: DiagnosticStage::CanonicalObjectModel,
        category: categories::CANONICAL_OBJECT_MODEL.to_owned(),
        message: message.into(),
        object_ref: None,
        remediation: None,
    }
}

fn validate_pipeline_identity(contract: &PipelineContract, report: &mut ValidationReport) {
    if contract.dpcs_version.trim().is_empty() {
        report.push(
            com_error("DPCS-COM-001", "dpcsVersion must not be empty")
                .with_object_ref("dpcsVersion")
                .with_remediation("Set dpcsVersion to a supported DPCS specification version"),
        );
    }

    if contract.id.trim().is_empty() {
        report.push(
            com_error("DPCS-COM-002", "pipeline identity id must not be empty")
                .with_object_ref("id")
                .with_remediation("Provide a stable pipeline identity"),
        );
    }

    if !is_present_version(&contract.version) {
        report.push(
            com_error(
                "DPCS-COM-003",
                "pipeline contract version must not be empty",
            )
            .with_object_ref("version")
            .with_remediation("Provide a pipeline contract version"),
        );
    }
}

fn validate_identity_catalog(contract: &PipelineContract, report: &mut ValidationReport) {
    let catalog = IdentityCatalog::from_contract(contract);

    for entry in catalog.entries_with_missing_ids() {
        report.push(
            com_error(
                "DPCS-COM-004",
                format!(
                    "{} object must have a stable non-empty identifier",
                    entry.kind
                ),
            )
            .with_object_ref(entry.path.as_str())
            .with_remediation("Provide a unique non-empty identifier"),
        );
    }

    for (kind, ids) in catalog.duplicate_ids_by_kind() {
        for id in ids {
            let paths = catalog.paths_for_kind_and_id(kind, id.as_str());
            let object_ref = paths
                .first()
                .map(|path| path.as_str().to_owned())
                .unwrap_or_else(|| format!("{kind}.{id}"));
            report.push(
                com_error(
                    "DPCS-COM-005",
                    format!("duplicate {} identifier `{}`", kind, id),
                )
                .with_object_ref(object_ref)
                .with_remediation("Ensure identifiers are unique within their object kind"),
            );
        }
    }
}

fn validate_interface_port_uniqueness(contract: &PipelineContract, report: &mut ValidationReport) {
    if contract.interface.has_unique_port_ids() {
        return;
    }

    let mut seen = BTreeSet::new();
    for port in contract.interface.all_ports() {
        if port.id.trim().is_empty() {
            continue;
        }
        if !seen.insert(port.id.as_str()) {
            report.push(
                com_error(
                    "DPCS-COM-013",
                    format!(
                        "duplicate interface port identifier `{}` across inputs and outputs",
                        port.id
                    ),
                )
                .with_object_ref(format!("interface.port.{}", port.id))
                .with_remediation(
                    "Ensure interface input and output identifiers are unique across the interface",
                ),
            );
        }
    }
}

fn validate_interface_ports(contract: &PipelineContract, report: &mut ValidationReport) {
    for (index, port) in contract.interface.inputs.iter().enumerate() {
        let object_ref = if port.id.trim().is_empty() {
            format!("interface.inputs[{index}]")
        } else {
            format!("interface.inputs.{}", port.id)
        };

        if port.is_complete() {
            continue;
        }

        if port.name.as_ref().map_or(true, |v| v.trim().is_empty()) {
            report.push(
                com_error(
                    "DPCS-COM-006",
                    "interface input must declare an interface name",
                )
                .with_object_ref(object_ref.clone())
                .with_remediation("Set name to the interface name for this input port"),
            );
        }

        if port
            .contract_ref
            .as_ref()
            .map_or(true, |v| v.trim().is_empty())
        {
            report.push(
                com_error(
                    "DPCS-COM-007",
                    "interface input must declare a contract reference",
                )
                .with_object_ref(object_ref.clone())
                .with_remediation("Set contractRef to an external contract reference"),
            );
        }

        if port.purpose.as_ref().map_or(true, |v| v.trim().is_empty()) {
            report.push(
                com_error(
                    "DPCS-COM-008",
                    "interface input must declare a logical purpose",
                )
                .with_object_ref(object_ref)
                .with_remediation("Set purpose to describe the logical purpose of this input"),
            );
        }
    }

    for (index, port) in contract.interface.outputs.iter().enumerate() {
        let object_ref = if port.id.trim().is_empty() {
            format!("interface.outputs[{index}]")
        } else {
            format!("interface.outputs.{}", port.id)
        };

        if port.is_complete() {
            continue;
        }

        if port.name.as_ref().map_or(true, |v| v.trim().is_empty()) {
            report.push(
                com_error(
                    "DPCS-COM-009",
                    "interface output must declare an interface name",
                )
                .with_object_ref(object_ref.clone())
                .with_remediation("Set name to the interface name for this output port"),
            );
        }

        if port
            .contract_ref
            .as_ref()
            .map_or(true, |v| v.trim().is_empty())
        {
            report.push(
                com_error(
                    "DPCS-COM-010",
                    "interface output must declare a contract reference",
                )
                .with_object_ref(object_ref.clone())
                .with_remediation("Set contractRef to an external contract reference"),
            );
        }

        if port.purpose.as_ref().map_or(true, |v| v.trim().is_empty()) {
            report.push(
                com_error(
                    "DPCS-COM-011",
                    "interface output must declare a logical purpose",
                )
                .with_object_ref(object_ref)
                .with_remediation("Set purpose to describe the logical purpose of this output"),
            );
        }
    }
}

fn validate_extension_keys(contract: &PipelineContract, report: &mut ValidationReport) {
    for key in contract.extensions.keys() {
        if is_reserved_root_field(key) {
            report.push(
                com_error(
                    "DPCS-COM-012",
                    format!("extension key `{key}` collides with a reserved root field"),
                )
                .with_object_ref(key.clone())
                .with_remediation("Use a namespaced extension key such as `x-vendor.field`"),
            );
        }
    }
}
