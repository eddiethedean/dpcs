//! Command-line interface for the `dpcs` binary.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::diagnostics::{Severity, ValidationReport};
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
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

fn execute(cli: Cli) -> Result<u8, crate::Error> {
    match cli.command {
        Commands::Version => {
            println!("dpcs {VERSION}");
            Ok(EXIT_OK)
        }
        Commands::Validate { path, json, strict } => {
            let contract = parser::parse_file(&path)?;
            let report = validate(&contract);
            print_report(&report, json);
            Ok(exit_code_for_report(&report, strict))
        }
        Commands::Inspect { path, json } => {
            let contract = parser::parse_file(&path)?;
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
                valid: contract.validate().is_valid(),
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
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
            }
            Ok(EXIT_OK)
        }
        Commands::Diagnostics { path, json } => {
            let contract = parser::parse_file(&path)?;
            let report = validate(&contract);
            print_report(&report, json);
            Ok(exit_code_for_report(&report, false))
        }
        Commands::Graph { path, json } => {
            let contract = parser::parse_file(&path)?;
            let plan = plan::plan(&contract);
            if json {
                let payload = GraphPayload {
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
                    step_order: plan.step_order,
                };
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
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
                if !plan.step_order.is_empty() {
                    println!("stepOrder: {}", plan.step_order.join(", "));
                }
            }
            Ok(EXIT_OK)
        }
    }
}

fn print_report(report: &ValidationReport, json: bool) {
    if json {
        match serde_json::to_string_pretty(report) {
            Ok(payload) => println!("{payload}"),
            Err(err) => eprintln!("error: failed to serialize diagnostics: {err}"),
        }
        return;
    }

    if report.diagnostics.is_empty() {
        println!("valid: no diagnostics");
        return;
    }

    for diagnostic in &report.diagnostics {
        let object = diagnostic
            .object_ref
            .as_deref()
            .map(|value| format!(" @ {value}"))
            .unwrap_or_default();
        println!(
            "{} {}: {}{} — {}",
            diagnostic.severity, diagnostic.id, diagnostic.category, object, diagnostic.message
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
}

fn exit_code_for_report(report: &ValidationReport, strict: bool) -> u8 {
    if report.error_count() > 0 {
        return EXIT_VALIDATION;
    }
    if strict && report.warning_count() > 0 {
        return EXIT_VALIDATION;
    }
    if report
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error)
    {
        EXIT_VALIDATION
    } else {
        EXIT_OK
    }
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
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphPayload {
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
