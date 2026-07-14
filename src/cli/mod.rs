//! Command-line interface for the `dpcs` binary.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::diagnostics::ValidationReport;
use crate::error::Error;
use crate::parser;
use crate::plan;
use crate::{validate, VERSION};

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
    /// Print the toolkit version.
    Version,
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
        Commands::Version => {
            println!("dpcs {VERSION}");
            Ok(EXIT_OK)
        }
        Commands::Validate { path, json, strict } => {
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    print_report(&report, json)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let report = validate(&contract);
            print_report(&report, json)?;
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
                valid,
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
                println!("valid: {}", summary.valid);
                if let Some(order) = &summary.step_order {
                    if !order.is_empty() {
                        println!("stepOrder: {}", order.join(", "));
                    }
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
            print_report(&report, json)?;
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
            let step_order = match &plan_result {
                plan::PlanResult::Ok(planned) => planned.step_order.clone(),
                plan::PlanResult::Err(_) => {
                    contract.steps.iter().map(|step| step.id.clone()).collect()
                }
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
                if !step_order.is_empty() {
                    println!("stepOrder: {}", step_order.join(", "));
                }
                if let plan::PlanResult::Err(report) = &plan_result {
                    println!("planning: refused ({} errors)", report.error_count());
                }
            }
            Ok(EXIT_OK)
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
        "summary: {} error(s), {} warning(s)",
        report.error_count(),
        report.warning_count()
    );
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
    valid: bool,
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
    step_order: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphEdgeView {
    from: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}
