//! Command-line interface for the `dpcs` binary.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::diagnostics::{DiagnosticReport, ValidationReport};
use crate::error::Error;
use crate::parser;
use crate::plan;
use crate::DPCS_SPEC_VERSION;
use crate::{
    apply_profile_to_contract, bind, compare_contracts, evaluate, parse_target, toolkit_claim,
    validate, validate_conformance_profile, validate_registry, write_bundle, BindingResult,
    CapabilityProfile, CapabilityResult, ConformanceProfile, Registry, VERSION,
};

/// Exit code for successful validation.
pub const EXIT_OK: u8 = 0;
/// Exit code for validation errors.
pub const EXIT_VALIDATION: u8 = 1;
/// Exit code for parse or I/O failures.
pub const EXIT_FAILURE: u8 = 2;

/// DPCS command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "dpcs",
    version = VERSION,
    about = "Data Pipeline Contract Standard (DPCS) toolkit",
    long_about = "Parse, inspect, and validate DPCS Pipeline Contracts.\n\nSPEC.md is the authoritative source of truth for DPCS semantics."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate a Pipeline Contract document.
    Validate {
        /// Path to a `.yaml`, `.yml`, or `.json` document.
        path: PathBuf,
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
        /// Treat warnings as errors.
        #[arg(long)]
        strict: bool,
        /// Optional conformance profile constraining extensions/security/governance.
        #[arg(long)]
        profile: Option<PathBuf>,
    },
    /// Inspect a Pipeline Contract and print a summary.
    Inspect {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Emit the summary as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Emit diagnostics for a Pipeline Contract.
    Diagnostics {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print the pipeline graph edges.
    Graph {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Emit the graph as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Evaluate a capability profile against a planned Pipeline Contract.
    Capabilities {
        /// Path to an orchestrator capability profile (`.yaml`, `.yml`, or `.json`).
        profile: PathBuf,
        /// Path to a Pipeline Contract used to build the plan under evaluation.
        #[arg(long)]
        plan: PathBuf,
        /// Emit the capability report (or diagnostics) as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Bind a Pipeline Contract to an orchestrator target (scaffold artifacts).
    Bind {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Path to an orchestrator capability profile.
        #[arg(long)]
        profile: PathBuf,
        /// Target orchestrator: airflow, dagster, prefect, temporal, kubernetes (alias: k8s).
        ///
        /// `temporal` and `kubernetes` are experimental.
        #[arg(long)]
        target: String,
        /// Directory to write generated artifacts (default: `./dpcs-bind-<target>/`).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Emit the binding bundle (or diagnostics) as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Compare two Pipeline Contracts for semantic compatibility.
    Compatibility {
        /// Baseline Pipeline Contract.
        baseline: PathBuf,
        /// Candidate Pipeline Contract.
        candidate: PathBuf,
        /// Emit the compatibility report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Registry document operations.
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Conformance profile / claim operations.
    Conformance {
        #[command(subcommand)]
        command: ConformanceCommands,
    },
    /// Print the toolkit version.
    Version {
        /// Emit version and conformance claim as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum RegistryCommands {
    /// Validate a registry document.
    Validate {
        /// Path to a registry YAML/JSON document.
        path: PathBuf,
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ConformanceCommands {
    /// Validate a conformance profile document.
    Validate {
        /// Path to a conformance profile YAML/JSON document.
        path: PathBuf,
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Run the CLI and return a process exit code.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(code) => ExitCode::from(code),
        Err(err) => handle_cli_error(err, false),
    }
}

fn handle_cli_error(err: Error, json: bool) -> ExitCode {
    if let Some(report) = err.invalid_document_report() {
        if let Err(print_err) = print_report(report, json) {
            eprintln!("error: {print_err}");
            return ExitCode::from(EXIT_FAILURE);
        }
        return ExitCode::from(EXIT_FAILURE);
    }

    eprintln!("error: {err}");
    ExitCode::from(EXIT_FAILURE)
}

fn execute(cli: Cli) -> Result<u8, Error> {
    match cli.command {
        Commands::Version { json } => {
            let claim = toolkit_claim();
            if json {
                let payload = VersionPayload {
                    version: VERSION.to_owned(),
                    dpcs_spec_version: DPCS_SPEC_VERSION.to_owned(),
                    conformance: claim,
                };
                let payload = serde_json::to_string_pretty(&payload).map_err(|err| {
                    Error::Serialization(format!("failed to serialize version payload: {err}"))
                })?;
                println!("{payload}");
            } else {
                println!("dpcs {VERSION}");
                println!("dpcsSpecVersion: {DPCS_SPEC_VERSION}");
                println!(
                    "conformanceLevels: {}",
                    claim
                        .levels
                        .iter()
                        .map(|level| level.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            Ok(EXIT_OK)
        }
        Commands::Validate {
            path,
            json,
            strict,
            profile,
        } => {
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let mut report = validate(&contract);
            if let Some(profile_path) = profile {
                let profile = match ConformanceProfile::from_file(&profile_path) {
                    Ok(profile) => profile,
                    Err(Error::InvalidDocument { report }) => {
                        print_report(&report, json)?;
                        return Ok(EXIT_FAILURE);
                    }
                    Err(err) => return Err(err),
                };
                let profile_report = validate_conformance_profile(&profile);
                if !profile_report.is_valid() {
                    print_report(&profile_report, json)?;
                    return Ok(EXIT_VALIDATION);
                }
                report.extend(apply_profile_to_contract(&contract, &profile));
                report.sort_deterministic();
            }
            if json {
                print_diagnostic_report(&DiagnosticReport::from_validation(
                    report.clone(),
                    Some(contract.id.clone()),
                ))?;
            } else {
                print_report(&report, false)?;
            }
            Ok(exit_code_for_report(&report, strict))
        }
        Commands::Inspect { path, json } => {
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let valid = contract.validate().is_valid();
            let planned = plan::try_plan(&contract);
            let planning_refused = planned.is_none();
            let summary = InspectSummary {
                id: contract.id.clone(),
                name: contract.name.clone(),
                version: contract.version.clone(),
                dpcs_version: contract.dpcs_version.clone(),
                step_count: contract.steps.len(),
                edge_count: contract.graph.edges.len(),
                input_count: contract.interface.inputs.len(),
                output_count: contract.interface.outputs.len(),
                contract_reference_count: contract.contract_references.len(),
                data_flow_count: contract.data_flow.len(),
                control_flow_count: contract.control_flow.len(),
                scheduling_count: contract.scheduling.len(),
                quality_gate_count: contract.quality_gates.len(),
                failure_semantics_count: contract.failure_semantics.len(),
                has_execution: contract.execution.is_some(),
                has_lineage: contract.lineage.is_some(),
                valid,
                planning_refused,
                step_order: planned.as_ref().map(|plan| plan.step_order.clone()),
            };
            if json {
                let payload = serde_json::to_string_pretty(&summary).map_err(|err| {
                    Error::Serialization(format!("failed to serialize inspect summary: {err}"))
                })?;
                println!("{payload}");
            } else {
                println!("id: {}", summary.id);
                if let Some(name) = &summary.name {
                    println!("name: {name}");
                }
                println!("version: {}", summary.version);
                println!("dpcsVersion: {}", summary.dpcs_version);
                println!("steps: {}", summary.step_count);
                println!("edges: {}", summary.edge_count);
                println!("inputs: {}", summary.input_count);
                println!("outputs: {}", summary.output_count);
                println!("contractReferences: {}", summary.contract_reference_count);
                println!("dataFlow: {}", summary.data_flow_count);
                println!("controlFlow: {}", summary.control_flow_count);
                println!("scheduling: {}", summary.scheduling_count);
                println!("qualityGates: {}", summary.quality_gate_count);
                println!("failureSemantics: {}", summary.failure_semantics_count);
                println!("execution: {}", summary.has_execution);
                println!("lineage: {}", summary.has_lineage);
                println!("valid: {}", summary.valid);
                if let Some(order) = &summary.step_order {
                    println!("stepOrder: {}", order.join(", "));
                } else {
                    println!("planning: refused");
                }
            }
            Ok(EXIT_OK)
        }
        Commands::Diagnostics { path, json } => {
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let report = validate(&contract);
            if json {
                print_diagnostic_report(&DiagnosticReport::from_validation(
                    report.clone(),
                    Some(contract.id.clone()),
                ))?;
            } else {
                print_report(&report, false)?;
            }
            Ok(exit_code_for_report(&report, false))
        }
        Commands::Graph { path, json } => {
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let plan_result = plan::plan(&contract);
            let (step_order, planning_refused) = match &plan_result {
                plan::PlanResult::Ok(planned) => (Some(planned.step_order.clone()), false),
                plan::PlanResult::Err(_) => (None, true),
            };
            if json {
                let payload = GraphPayload {
                    entry_points: contract.graph.entry_points.clone(),
                    exit_points: contract.graph.exit_points.clone(),
                    edges: contract
                        .graph
                        .edges
                        .iter()
                        .map(|e| GraphEdgeView {
                            from: e.from.clone(),
                            to: e.to.clone(),
                            kind: e.kind.clone(),
                        })
                        .collect(),
                    step_order,
                    planning_refused,
                };
                let payload = serde_json::to_string_pretty(&payload).map_err(|err| {
                    Error::Serialization(format!("failed to serialize graph payload: {err}"))
                })?;
                println!("{payload}");
            } else {
                if !contract.graph.entry_points.is_empty() {
                    println!("entryPoints: {}", contract.graph.entry_points.join(", "));
                }
                if !contract.graph.exit_points.is_empty() {
                    println!("exitPoints: {}", contract.graph.exit_points.join(", "));
                }
                if contract.graph.edges.is_empty() {
                    println!("(no edges)");
                } else {
                    for edge in &contract.graph.edges {
                        match &edge.kind {
                            Some(kind) => println!("{} -> {} ({kind})", edge.from, edge.to),
                            None => println!("{} -> {}", edge.from, edge.to),
                        }
                    }
                }
                match &step_order {
                    Some(order) => println!("stepOrder: {}", order.join(", ")),
                    None => {
                        if let plan::PlanResult::Err(report) = &plan_result {
                            println!("planning: refused ({} errors)", report.error_count());
                        } else {
                            println!("planning: refused");
                        }
                    }
                }
            }
            Ok(EXIT_OK)
        }
        Commands::Capabilities {
            profile,
            plan: contract_path,
            json,
        } => {
            let profile = match CapabilityProfile::from_file(&profile) {
                Ok(profile) => profile,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let contract = match parser::parse_file(&contract_path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };

            let planned = match plan::plan(&contract) {
                plan::PlanResult::Ok(planned) => planned,
                plan::PlanResult::Err(report) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_VALIDATION);
                }
            };

            match evaluate(&planned, &profile) {
                CapabilityResult::Ok(report) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&*report).map_err(|err| {
                            Error::Serialization(format!(
                                "failed to serialize capability report: {err}"
                            ))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("profile: {}", report.profile_identity);
                        if let Some(contract_id) = &report.plan_contract_id {
                            println!("contractId: {contract_id}");
                        }
                        println!("satisfied: {}", report.satisfied.join(", "));
                        if !report.unsupported_optional.is_empty() {
                            println!(
                                "unsupportedOptional: {}",
                                report.unsupported_optional.join(", ")
                            );
                        }
                        for diagnostic in &report.diagnostics {
                            println!(
                                "{} {}: {} — {}",
                                diagnostic.severity,
                                diagnostic.id,
                                diagnostic.stage,
                                diagnostic.message
                            );
                        }
                        println!("match: ok");
                    }
                    Ok(EXIT_OK)
                }
                CapabilityResult::Err {
                    report,
                    diagnostics,
                } => {
                    if json {
                        let mut payload = (*report).clone();
                        payload.diagnostics = diagnostics.diagnostics.clone();
                        let payload = serde_json::to_string_pretty(&payload).map_err(|err| {
                            Error::Serialization(format!(
                                "failed to serialize capability report: {err}"
                            ))
                        })?;
                        println!("{payload}");
                    } else {
                        print_report(&diagnostics, false)?;
                        if !report.missing_mandatory.is_empty() {
                            println!("missingMandatory: {}", report.missing_mandatory.join(", "));
                        }
                    }
                    Ok(EXIT_VALIDATION)
                }
            }
        }
        Commands::Bind {
            path,
            profile,
            target,
            out,
            json,
        } => {
            let target = match parse_target(&target) {
                Ok(target) => target,
                Err(report) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_VALIDATION);
                }
            };
            let profile = match CapabilityProfile::from_file(&profile) {
                Ok(profile) => profile,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };

            let planned = match plan::plan(&contract) {
                plan::PlanResult::Ok(planned) => planned,
                plan::PlanResult::Err(report) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_VALIDATION);
                }
            };

            match bind(&planned, &profile, target) {
                BindingResult::Ok(bundle) => {
                    let out_dir = out.unwrap_or_else(|| {
                        PathBuf::from(format!("dpcs-bind-{}", bundle.target.as_str()))
                    });
                    if let Err(report) = write_bundle(&bundle, &out_dir) {
                        print_report(&report, json)?;
                        return Ok(EXIT_FAILURE);
                    }
                    if json {
                        let payload = serde_json::to_string_pretty(&*bundle).map_err(|err| {
                            Error::Serialization(format!(
                                "failed to serialize binding bundle: {err}"
                            ))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("target: {}", bundle.target);
                        if bundle.target.is_experimental() {
                            println!("experimental: true");
                        }
                        println!("contractId: {}", bundle.contract_id);
                        println!("contractVersion: {}", bundle.contract_version);
                        println!("profile: {}", bundle.profile_identity);
                        println!("out: {}", out_dir.display());
                        println!("files:");
                        for file in &bundle.files {
                            println!("  - {} ({})", file.relative_path, file.media_type);
                        }
                        println!("bind: ok");
                    }
                    Ok(EXIT_OK)
                }
                BindingResult::Err { diagnostics, .. } => {
                    print_report(&diagnostics, json)?;
                    Ok(EXIT_VALIDATION)
                }
            }
        }
        Commands::Compatibility {
            baseline,
            candidate,
            json,
        } => {
            let baseline_contract = match parser::parse_file(&baseline) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let candidate_contract = match parser::parse_file(&candidate) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let result = compare_contracts(&baseline_contract, &candidate_contract);
            let report = result.report();
            if json {
                let payload = serde_json::to_string_pretty(report).map_err(|err| {
                    Error::Serialization(format!("failed to serialize compatibility report: {err}"))
                })?;
                println!("{payload}");
            } else {
                println!(
                    "baseline: {}@{}",
                    report.baseline_id, report.baseline_version
                );
                println!(
                    "candidate: {}@{}",
                    report.candidate_id, report.candidate_version
                );
                println!("category: {}", report.category);
                for diagnostic in &report.diagnostics {
                    println!(
                        "{} {}: {} — {}",
                        diagnostic.severity, diagnostic.id, diagnostic.stage, diagnostic.message
                    );
                }
                println!(
                    "compatibility: {}",
                    if report.category.is_compatible() {
                        "ok"
                    } else {
                        "incompatible"
                    }
                );
            }
            Ok(if report.category.is_compatible() {
                EXIT_OK
            } else {
                EXIT_VALIDATION
            })
        }
        Commands::Registry {
            command: RegistryCommands::Validate { path, json },
        } => {
            let registry = match Registry::from_file(&path) {
                Ok(registry) => registry,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let report = validate_registry(&registry);
            print_report(&report, json)?;
            Ok(exit_code_for_report(&report, false))
        }
        Commands::Conformance {
            command: ConformanceCommands::Validate { path, json },
        } => {
            let profile = match ConformanceProfile::from_file(&path) {
                Ok(profile) => profile,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let report = validate_conformance_profile(&profile);
            print_report(&report, json)?;
            Ok(exit_code_for_report(&report, false))
        }
    }
}

fn print_report(report: &ValidationReport, json: bool) -> Result<(), Error> {
    if json {
        let payload = serde_json::to_string_pretty(report).map_err(|err| {
            Error::Serialization(format!("failed to serialize diagnostics: {err}"))
        })?;
        println!("{payload}");
        return Ok(());
    }

    if report.diagnostics.is_empty() {
        println!("valid: no diagnostics");
        return Ok(());
    }

    for diagnostic in &report.diagnostics {
        let object = diagnostic
            .object_ref
            .as_deref()
            .map(|value| format!(" @ {value}"))
            .unwrap_or_default();
        let location = diagnostic
            .source_location
            .as_deref()
            .map(|value| format!(" ({value})"))
            .unwrap_or_default();
        println!(
            "{} {}: {}{} — {}{}",
            diagnostic.severity,
            diagnostic.id,
            diagnostic.stage,
            object,
            diagnostic.message,
            location
        );
        if let Some(remediation) = &diagnostic.remediation {
            println!("  remediation: {remediation}");
        }
    }

    println!(
        "summary: {} error(s), {} warning(s), {} information(s)",
        report.error_count(),
        report.warning_count(),
        report.information_count()
    );
    Ok(())
}

fn print_diagnostic_report(report: &DiagnosticReport) -> Result<(), Error> {
    let payload = serde_json::to_string_pretty(report).map_err(|err| {
        Error::Serialization(format!("failed to serialize diagnostic report: {err}"))
    })?;
    println!("{payload}");
    Ok(())
}

fn exit_code_for_report(report: &ValidationReport, strict: bool) -> u8 {
    if report.error_count() > 0 {
        return EXIT_VALIDATION;
    }
    if strict && report.warning_count() > 0 {
        return EXIT_VALIDATION;
    }
    EXIT_OK
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectSummary {
    id: String,
    name: Option<String>,
    version: String,
    dpcs_version: String,
    step_count: usize,
    edge_count: usize,
    input_count: usize,
    output_count: usize,
    contract_reference_count: usize,
    data_flow_count: usize,
    control_flow_count: usize,
    scheduling_count: usize,
    quality_gate_count: usize,
    failure_semantics_count: usize,
    has_execution: bool,
    has_lineage: bool,
    valid: bool,
    planning_refused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    step_order: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphPayload {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    entry_points: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exit_points: Vec<String>,
    edges: Vec<GraphEdgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    step_order: Option<Vec<String>>,
    planning_refused: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphEdgeView {
    from: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionPayload {
    version: String,
    dpcs_spec_version: String,
    conformance: crate::ConformanceClaim,
}
