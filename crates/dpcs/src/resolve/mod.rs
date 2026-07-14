//! Contract reference resolution (SPEC Ch 7).
//!
//! Resolves declared [`ContractReference`] locations from the filesystem,
//! unpacked `.dpcspkg` roots, and optionally a registry HTTP client. Nested
//! DPCS Pipeline Contracts are loaded into the Canonical Object Model;
//! ODCS/DTCS and other types are resolved to an on-disk (or fetched) path only.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    ContractReference, PipelineContract, PipelineLineage, PipelineProvenance, PipelineStep,
};
use crate::package::PackageLayout;
use crate::parser;

/// Options controlling how Contract References are resolved.
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    /// Directory used as the root for relative `location` paths.
    ///
    /// Typically the parent directory of the Pipeline Contract document.
    pub base_dir: Option<PathBuf>,
    /// Unpacked package roots searched for artifact paths.
    pub package_roots: Vec<PathBuf>,
    /// Optional registry base URL (`registry-client` feature).
    #[cfg(feature = "registry-client")]
    pub registry_base_url: Option<String>,
}

/// Maximum nesting depth for recursive DPCS pipeline resolution.
pub const MAX_NESTING_DEPTH: usize = 8;

impl ResolveOptions {
    /// Build options rooted at the parent of `document_path`.
    pub fn from_document_path(document_path: impl AsRef<Path>) -> Self {
        let parent = document_path
            .as_ref()
            .parent()
            .map(Path::to_path_buf)
            .filter(|p| !p.as_os_str().is_empty());
        Self {
            base_dir: parent,
            package_roots: Vec::new(),
            #[cfg(feature = "registry-client")]
            registry_base_url: None,
        }
    }

    /// Default options for library `plan()` / `bind_contract()` (SPEC Ch 7).
    ///
    /// Uses the process current directory as `base_dir`. Prefer
    /// [`Self::from_document_path`] when locations are relative to a contract file.
    pub fn default_for_planning() -> Self {
        Self {
            base_dir: std::env::current_dir().ok(),
            package_roots: Vec::new(),
            #[cfg(feature = "registry-client")]
            registry_base_url: None,
        }
    }

    /// Add an unpacked package root to the search path.
    pub fn with_package_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.package_roots.push(root.into());
        self
    }
}

/// A nested Pipeline Contract loaded via a step `contractRef`.
#[derive(Debug, Clone, PartialEq)]
pub struct NestedPipeline {
    /// Parent step identifier in the enclosing contract.
    pub parent_step_id: String,
    /// Stable reference id or location used to load the nested contract.
    pub contract_ref: String,
    /// Loaded nested Pipeline Contract (own identity/interface preserved).
    pub contract: PipelineContract,
    /// Recursively resolved child nested pipelines.
    pub children: Vec<NestedPipeline>,
}

/// Outcome of resolving Contract References for a Pipeline Contract.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Map of contract-reference id → absolute (or canonical) resolved path.
    pub locations: BTreeMap<String, PathBuf>,
    /// Nested DPCS pipelines discovered via step `contractRef`s.
    pub nested: Vec<NestedPipeline>,
    /// Diagnostics produced during resolution.
    pub report: ValidationReport,
}

impl ResolutionResult {
    /// Returns whether resolution produced no errors.
    pub fn is_ok(&self) -> bool {
        self.report.is_valid()
    }
}

/// Resolve Contract References declared on `contract`.
///
/// Locations are sought under `base_dir`, then package roots, then registry
/// (when configured). Nested DPCS documents (reference type `dpcs` / `pipeline`
/// / `dpcs:pipeline`) are parsed so identity and interfaces can be preserved.
/// Nested DPCS graphs are resolved recursively with cycle detection.
pub fn resolve_contract_references(
    contract: &PipelineContract,
    options: &ResolveOptions,
) -> ResolutionResult {
    let mut report = ValidationReport::new();
    let mut ancestors = std::collections::BTreeSet::new();
    ancestors.insert(contract.id.clone());
    let (locations, nested) = resolve_level(contract, options, &mut ancestors, 0, &mut report);
    report.sort_deterministic();
    ResolutionResult {
        locations,
        nested,
        report,
    }
}

fn resolve_level(
    contract: &PipelineContract,
    options: &ResolveOptions,
    ancestors: &mut std::collections::BTreeSet<String>,
    depth: usize,
    report: &mut ValidationReport,
) -> (BTreeMap<String, PathBuf>, Vec<NestedPipeline>) {
    let mut locations = BTreeMap::new();
    let mut nested = Vec::new();

    let by_id: BTreeMap<&str, &ContractReference> = contract
        .contract_references
        .iter()
        .map(|r| (r.id.as_str(), r))
        .collect();

    for (index, reference) in contract.contract_references.iter().enumerate() {
        let require_presence = is_dpcs_reference_type(&reference.reference_type);
        match locate_reference(reference, options, report, index, require_presence) {
            Some(path) => {
                locations.insert(reference.id.clone(), path.clone());
                if require_presence {
                    if let Err(message) = load_nested_contract(&path) {
                        report.push(
                            Diagnostic::error(
                                "DPCS-REF-007",
                                categories::REFERENCE,
                                format!(
                                    "contract reference `{}` resolves to `{}` but could not be loaded as a Pipeline Contract: {message}",
                                    reference.id,
                                    path.display()
                                ),
                            )
                            .with_object_ref(format!("contractReferences[{index}].location"))
                            .with_remediation(
                                "Ensure the location points to a valid DPCS YAML/JSON document",
                            ),
                        );
                    }
                }
            }
            None => {
                // Companion ODCS/DTCS locations may be external. Nested DPCS errors
                // are recorded inside locate_reference when require_presence.
            }
        }
    }

    for step in &contract.steps {
        if let Some(nested_pipeline) = resolve_step_nested(
            contract, step, &by_id, &locations, options, ancestors, depth, report,
        ) {
            nested.push(nested_pipeline);
        }
    }

    (locations, nested)
}

/// Validate a contract and merge deep reference-resolution diagnostics.
pub fn validate_resolved(
    contract: &PipelineContract,
    options: &ResolveOptions,
) -> ValidationReport {
    let mut report = crate::validation::validate(contract);
    let resolution = resolve_contract_references(contract, options);
    report.extend(resolution.report);
    report.sort_deterministic();
    report
}

/// Apply nested parent–child provenance onto a lineage value.
pub fn apply_nested_provenance(
    lineage: &mut Option<PipelineLineage>,
    parent_id: &str,
    nested: &[NestedPipeline],
) {
    if nested.is_empty() {
        return;
    }
    let lineage = lineage.get_or_insert_with(PipelineLineage::default);
    let provenance = lineage
        .provenance
        .get_or_insert_with(PipelineProvenance::default);
    if provenance.originating.is_none() {
        provenance.originating = Some(parent_id.to_owned());
    }
    collect_nested_ids(nested, &mut provenance.nested);
    provenance.nested.sort();
    provenance.nested.dedup();
}

fn collect_nested_ids(nested: &[NestedPipeline], out: &mut Vec<String>) {
    for child in nested {
        let child_id = child.contract.id.clone();
        if !out.contains(&child_id) {
            out.push(child_id);
        }
        collect_nested_ids(&child.children, out);
    }
}

/// Stamp parent identifiers onto nested contracts' lineage for preservation.
pub fn stamp_nested_parents(nested: &mut [NestedPipeline], parent_id: &str) {
    for child in nested.iter_mut() {
        let lineage = child
            .contract
            .lineage
            .get_or_insert_with(PipelineLineage::default);
        let provenance = lineage
            .provenance
            .get_or_insert_with(PipelineProvenance::default);
        if !provenance.parents.contains(&parent_id.to_owned()) {
            provenance.parents.push(parent_id.to_owned());
        }
        provenance.parents.sort();
        provenance.parents.dedup();
        let child_id = child.contract.id.clone();
        stamp_nested_parents(&mut child.children, &child_id);
    }
}

fn is_dpcs_reference_type(reference_type: &str) -> bool {
    let t = reference_type.trim().to_ascii_lowercase();
    matches!(t.as_str(), "dpcs" | "pipeline" | "dpcs:pipeline" | "nested")
}

fn is_nested_step(step: &PipelineStep) -> bool {
    let t = step.step_type.trim().to_ascii_lowercase();
    matches!(
        t.as_str(),
        "dpcs:pipeline" | "nested" | "pipeline" | "dpcs:nested"
    )
}

fn locate_reference(
    reference: &ContractReference,
    options: &ResolveOptions,
    report: &mut ValidationReport,
    index: usize,
    require_presence: bool,
) -> Option<PathBuf> {
    let location = reference.location.trim();
    if location.is_empty() {
        return None;
    }

    if let Some(path) = try_local_path(location, options) {
        return Some(path);
    }

    for root in &options.package_roots {
        if let Ok(layout) = PackageLayout::open(root) {
            if let Some(path) = layout.resolve_path(&reference.id, reference.version.as_deref()) {
                if path.is_file() {
                    return Some(path);
                }
            }
            // Also try location relative to package root.
            let candidate = root.join(location);
            if candidate.is_file() {
                return Some(canonicalize_or(candidate));
            }
        }
    }

    #[cfg(feature = "registry-client")]
    if let Some(base) = &options.registry_base_url {
        if let Some(path) = try_registry_fetch(base, reference, report, index) {
            return Some(path);
        }
    }

    // Companion ODCS/DTCS (and similar) locations may be external; absence is not
    // an error. Nested DPCS Pipeline Contracts must resolve to a readable document.
    if require_presence {
        report.push(
            Diagnostic::error(
                "DPCS-REF-007",
                categories::REFERENCE,
                format!(
                    "contract reference `{}` location `{}` could not be resolved",
                    reference.id, location
                ),
            )
            .with_object_ref(format!("contractReferences[{index}].location"))
            .with_remediation(
                "Provide a readable nested DPCS document under the contract directory or package root",
            ),
        );
    }
    None
}

fn try_local_path(location: &str, options: &ResolveOptions) -> Option<PathBuf> {
    let as_path = Path::new(location);
    if as_path.is_absolute() && as_path.is_file() {
        return Some(canonicalize_or(as_path.to_path_buf()));
    }
    if let Some(base) = &options.base_dir {
        let candidate = base.join(location);
        if candidate.is_file() {
            return Some(canonicalize_or(candidate));
        }
    }
    // Relative to process CWD as a last local attempt when base_dir is set misses.
    if as_path.is_file() {
        return Some(canonicalize_or(as_path.to_path_buf()));
    }
    None
}

fn canonicalize_or(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn load_nested_contract(path: &Path) -> Result<PipelineContract, String> {
    parser::parse_file(path).map_err(|err| err.to_string())
}

#[allow(clippy::too_many_arguments)]
fn resolve_step_nested(
    parent: &PipelineContract,
    step: &PipelineStep,
    by_id: &BTreeMap<&str, &ContractReference>,
    locations: &BTreeMap<String, PathBuf>,
    options: &ResolveOptions,
    ancestors: &mut std::collections::BTreeSet<String>,
    depth: usize,
    report: &mut ValidationReport,
) -> Option<NestedPipeline> {
    let contract_ref = step.contract_ref.as_deref()?;
    let reference = by_id.get(contract_ref).copied();
    let wants_nested = reference
        .map(|r| is_dpcs_reference_type(&r.reference_type))
        .unwrap_or_else(|| is_nested_step(step));
    if !wants_nested {
        return None;
    }

    if depth >= MAX_NESTING_DEPTH {
        report.push(
            Diagnostic::error(
                "DPCS-REF-008",
                categories::REFERENCE,
                format!(
                    "nested pipeline depth exceeds limit ({MAX_NESTING_DEPTH}) at step `{}`",
                    step.id
                ),
            )
            .with_object_ref(format!("steps.{}.contractRef", step.id)),
        );
        return None;
    }

    let path = if let Some(p) = locations.get(contract_ref) {
        p.clone()
    } else if let Some(reference) = reference {
        locate_reference(
            reference,
            options,
            report,
            parent
                .contract_references
                .iter()
                .position(|r| r.id == reference.id)
                .unwrap_or(0),
            true,
        )?
    } else if looks_like_path(contract_ref) {
        try_local_path(contract_ref, options)?
    } else {
        return None;
    };

    match load_nested_contract(&path) {
        Ok(nested_contract) => {
            if nested_contract.id.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-008",
                        categories::REFERENCE,
                        format!(
                            "nested pipeline for step `{}` is missing a stable identity",
                            step.id
                        ),
                    )
                    .with_object_ref(format!("steps.{}.contractRef", step.id)),
                );
                return None;
            }
            if ancestors.contains(&nested_contract.id) {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-008",
                        categories::REFERENCE,
                        format!(
                            "nested pipeline cycle detected involving `{}` (step `{}`)",
                            nested_contract.id, step.id
                        ),
                    )
                    .with_object_ref(format!("steps.{}.contractRef", step.id))
                    .with_remediation("Remove cyclic nested Pipeline Contract references"),
                );
                return None;
            }
            let nested_report = crate::validation::validate(&nested_contract);
            if !nested_report.is_valid() {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-008",
                        categories::REFERENCE,
                        format!(
                            "nested pipeline `{}` referenced by step `{}` failed validation",
                            nested_contract.id, step.id
                        ),
                    )
                    .with_object_ref(format!("steps.{}.contractRef", step.id))
                    .with_related(nested_report.errors().map(|d| d.id.clone()))
                    .with_remediation("Fix validation errors in the nested Pipeline Contract"),
                );
                report.extend(nested_report);
                return None;
            }

            let child_opts = ResolveOptions {
                base_dir: path.parent().map(Path::to_path_buf),
                package_roots: options.package_roots.clone(),
                #[cfg(feature = "registry-client")]
                registry_base_url: options.registry_base_url.clone(),
            };
            ancestors.insert(nested_contract.id.clone());
            let (_child_locations, children) =
                resolve_level(&nested_contract, &child_opts, ancestors, depth + 1, report);
            ancestors.remove(&nested_contract.id);

            Some(NestedPipeline {
                parent_step_id: step.id.clone(),
                contract_ref: contract_ref.to_owned(),
                contract: nested_contract,
                children,
            })
        }
        Err(message) => {
            report.push(
                Diagnostic::error(
                    "DPCS-REF-007",
                    categories::REFERENCE,
                    format!(
                        "step `{}` nested contract at `{}` could not be loaded: {message}",
                        step.id,
                        path.display()
                    ),
                )
                .with_object_ref(format!("steps.{}.contractRef", step.id)),
            );
            None
        }
    }
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/')
}

#[cfg(feature = "registry-client")]
fn try_registry_fetch(
    base: &str,
    reference: &ContractReference,
    report: &mut ValidationReport,
    index: usize,
) -> Option<PathBuf> {
    use crate::registry_net::RegistryClient;

    let mut client = match RegistryClient::new(base) {
        Ok(client) => client,
        Err(err) => {
            report.push(
                Diagnostic::error(
                    "DPCS-REF-007",
                    categories::REFERENCE,
                    format!(
                        "contract reference `{}`: invalid registry base URL: {err}",
                        reference.id
                    ),
                )
                .with_object_ref(format!("contractReferences[{index}].location")),
            );
            return None;
        }
    };
    match client.fetch_content(&reference.id, reference.version.as_deref()) {
        Ok(body) => {
            let tmp = std::env::temp_dir().join(format!(
                "dpcs-resolve-{}-{}.yaml",
                reference.id.replace('/', "_"),
                reference.version.clone().unwrap_or_else(|| "latest".into())
            ));
            if let Err(err) = fs::write(&tmp, body.as_bytes()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-007",
                        categories::REFERENCE,
                        format!(
                            "contract reference `{}`: failed to cache registry artifact: {err}",
                            reference.id
                        ),
                    )
                    .with_object_ref(format!("contractReferences[{index}].location")),
                );
                return None;
            }
            Some(tmp)
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn resolves_relative_nested_pipeline() {
        let dir = tempfile::tempdir().unwrap();
        let nested_path = dir.path().join("child.dpcs.yaml");
        fs::write(
            &nested_path,
            r#"
dpcsVersion: "1.0.0"
id: "child.pipe"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps: []
graph:
  edges: []
"#,
        )
        .unwrap();

        let parent = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "parent.pipe"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
contractReferences:
  - id: child
    type: dpcs
    location: child.dpcs.yaml
steps:
  - id: nest
    type: dpcs:pipeline
    contractRef: child
graph:
  edges: []
"#,
        )
        .unwrap();

        let opts = ResolveOptions {
            base_dir: Some(dir.path().to_path_buf()),
            package_roots: Vec::new(),
            #[cfg(feature = "registry-client")]
            registry_base_url: None,
        };
        let result = resolve_contract_references(&parent, &opts);
        assert!(result.is_ok(), "{:?}", result.report.diagnostics);
        assert_eq!(result.nested.len(), 1);
        assert_eq!(result.nested[0].contract.id, "child.pipe");
        assert!(result.nested[0].children.is_empty());
    }
}
