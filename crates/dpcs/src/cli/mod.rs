//! Command-line interface for the `dpcs` binary.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::diagnostics::{DiagnosticReport, ValidationReport};
use crate::error::Error;
use crate::parser;
use crate::plan;
use crate::report::{graph_view_from_contract, inspect_view_from_contract, ReportFormat};
use crate::DPCS_SPEC_VERSION;
use crate::{
    apply_profile_to_contract, bind, compare_contracts, evaluate, openapi_document, pack,
    parse_target, serve, toolkit_claim, unpack, validate, validate_conformance_profile,
    validate_package, validate_registry, write_bundle, write_document_schemas,
    write_openapi_documents, BindingResult, CapabilityProfile, CapabilityResult,
    ConformanceProfile, OpenApiKind, PackageLayout, PublishRequest, Registry, RegistryCache,
    RegistryClient, RegistryClientError, ServeOptions, VERSION,
};

mod output;
#[cfg(feature = "tui")]
mod tui;

use output::{
    emit_capability, emit_compatibility, emit_diagnostic_report, emit_graph, emit_inspect,
    emit_validation, EmitOpts,
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
        /// Emit diagnostics as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
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
        /// Emit the summary as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
        /// Launch the interactive TUI inspector (requires `tui` feature).
        #[arg(long)]
        tui: bool,
    },
    /// Emit diagnostics for a Pipeline Contract.
    Diagnostics {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Emit diagnostics as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Print the pipeline graph edges.
    Graph {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
        /// Emit the graph as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`, `mermaid`, `dot`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Evaluate a capability profile against a planned Pipeline Contract.
    Capabilities {
        /// Path to an orchestrator capability profile (`.yaml`, `.yml`, or `.json`).
        profile: PathBuf,
        /// Path to a Pipeline Contract used to build the plan under evaluation.
        #[arg(long)]
        plan: PathBuf,
        /// Emit the capability report (or diagnostics) as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
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
        /// Emit the compatibility report as JSON (alias for `--format json`).
        #[arg(long)]
        json: bool,
        /// Output format: `text`, `json`, `markdown`, `html`.
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        /// Write report to a file instead of stdout.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Interactive TUI inspector for one Pipeline Contract.
    #[cfg(feature = "tui")]
    Tui {
        /// Path to a Pipeline Contract document.
        path: PathBuf,
    },
    /// Registry document and network operations.
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Conformance profile / claim operations.
    Conformance {
        #[command(subcommand)]
        command: ConformanceCommands,
    },
    /// Pipeline package (.dpcspkg) operations.
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
    /// Emit JSON Schema / OpenAPI artifacts.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
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
    /// Serve a file-backed reference registry HTTP API.
    Serve {
        /// Directory containing registry.yaml and artifact files.
        #[arg(long)]
        root: PathBuf,
        /// Bind address (host:port).
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
        /// Optional bearer token for mutating operations.
        #[arg(long)]
        token: Option<String>,
    },
    /// Pull the remote registry document.
    Pull {
        /// Registry base URL.
        #[arg(long)]
        url: String,
        /// Optional bearer token.
        #[arg(long)]
        token: Option<String>,
        /// Optional on-disk client cache directory.
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Lookup an artifact by id.
    Lookup {
        /// Registry base URL.
        #[arg(long)]
        url: String,
        /// Artifact id.
        id: String,
        /// Optional artifact version.
        #[arg(long)]
        version: Option<String>,
        /// Optional bearer token.
        #[arg(long)]
        token: Option<String>,
        /// Optional on-disk client cache directory.
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Publish an artifact metadata document (YAML/JSON RegisteredArtifact).
    Publish {
        /// Registry base URL.
        #[arg(long)]
        url: String,
        /// Path to RegisteredArtifact YAML/JSON (optional content via `--content`).
        path: PathBuf,
        /// Optional payload file to store as content.
        #[arg(long)]
        content: Option<PathBuf>,
        /// Optional bearer token.
        #[arg(long)]
        token: Option<String>,
        /// Optional on-disk client cache directory.
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Deprecate an artifact.
    Deprecate {
        /// Registry base URL.
        #[arg(long)]
        url: String,
        /// Artifact id.
        id: String,
        /// Optional artifact version (defaults to latest listed).
        #[arg(long)]
        version: Option<String>,
        /// Optional bearer token.
        #[arg(long)]
        token: Option<String>,
        /// Optional on-disk client cache directory.
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Retire an artifact.
    Retire {
        /// Registry base URL.
        #[arg(long)]
        url: String,
        /// Artifact id.
        id: String,
        /// Optional artifact version (defaults to latest listed).
        #[arg(long)]
        version: Option<String>,
        /// Optional bearer token.
        #[arg(long)]
        token: Option<String>,
        /// Optional on-disk client cache directory.
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Clear or inspect the local registry client cache directory.
    Cache {
        /// Cache directory.
        #[arg(long)]
        dir: PathBuf,
        /// Clear the cache.
        #[arg(long)]
        clear: bool,
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

#[derive(Debug, Subcommand)]
enum PackageCommands {
    /// Validate a package directory.
    Validate {
        /// Path to an unpacked `.dpcspkg` directory.
        path: PathBuf,
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show package manifest summary.
    Show {
        /// Path to an unpacked `.dpcspkg` directory.
        path: PathBuf,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Pack a package directory (optionally to a zip archive).
    Pack {
        /// Path to an unpacked package directory.
        path: PathBuf,
        /// Optional output zip path.
        #[arg(long)]
        archive: Option<PathBuf>,
    },
    /// Unpack a zip archive or copy a package directory.
    Unpack {
        /// Source package directory or zip.
        source: PathBuf,
        /// Destination directory.
        dest: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum SchemaCommands {
    /// Write document JSON Schemas.
    Json {
        /// Output directory.
        #[arg(long, default_value = "schemas")]
        out: PathBuf,
    },
    /// Write OpenAPI documents.
    Openapi {
        /// documents | registry | all
        #[arg(long, default_value = "all")]
        kind: String,
        /// Output directory (or file when kind is documents|registry).
        #[arg(long, default_value = "schemas")]
        out: PathBuf,
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
        #[cfg(feature = "tui")]
        Commands::Tui { path } => run_inspect_tui(&path),
        Commands::Validate {
            path,
            json,
            format,
            out,
            strict,
            profile,
        } => {
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let mut report = validate(&contract);
            if let Some(profile_path) = profile {
                let profile = match ConformanceProfile::from_file(&profile_path) {
                    Ok(profile) => profile,
                    Err(Error::InvalidDocument { report }) => {
                        emit_validation(&opts, &report)?;
                        return Ok(EXIT_FAILURE);
                    }
                    Err(err) => return Err(err),
                };
                let profile_report = validate_conformance_profile(&profile);
                if !profile_report.is_valid() {
                    emit_validation(&opts, &profile_report)?;
                    return Ok(EXIT_VALIDATION);
                }
                report.extend(apply_profile_to_contract(&contract, &profile));
                report.sort_deterministic();
            }
            if opts.format == ReportFormat::Json {
                emit_diagnostic_report(
                    &opts,
                    &DiagnosticReport::from_validation(report.clone(), Some(contract.id.clone())),
                )?;
            } else {
                emit_validation(&opts, &report)?;
            }
            Ok(exit_code_for_report(&report, strict))
        }
        Commands::Inspect {
            path,
            json,
            format,
            out,
            tui,
        } => {
            if tui {
                return run_inspect_tui(&path);
            }
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let view = inspect_view_from_contract(&contract);
            emit_inspect(&opts, &view)?;
            Ok(EXIT_OK)
        }
        Commands::Diagnostics {
            path,
            json,
            format,
            out,
        } => {
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let report = validate(&contract);
            if opts.format == ReportFormat::Json {
                emit_diagnostic_report(
                    &opts,
                    &DiagnosticReport::from_validation(report.clone(), Some(contract.id.clone())),
                )?;
            } else {
                emit_validation(&opts, &report)?;
            }
            Ok(exit_code_for_report(&report, false))
        }
        Commands::Graph {
            path,
            json,
            format,
            out,
        } => {
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let contract = match parser::parse_file(&path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let view = graph_view_from_contract(&contract);
            emit_graph(&opts, &view)?;
            Ok(EXIT_OK)
        }
        Commands::Capabilities {
            profile,
            plan: contract_path,
            json,
            format,
            out,
        } => {
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let profile = match CapabilityProfile::from_file(&profile) {
                Ok(profile) => profile,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let contract = match parser::parse_file(&contract_path) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };

            let planned = match plan::plan(&contract) {
                plan::PlanResult::Ok(planned) => planned,
                plan::PlanResult::Err(report) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_VALIDATION);
                }
            };

            match evaluate(&planned, &profile) {
                CapabilityResult::Ok(report) => {
                    emit_capability(&opts, &report, true)?;
                    Ok(EXIT_OK)
                }
                CapabilityResult::Err {
                    report,
                    diagnostics,
                } => {
                    let mut merged = (*report).clone();
                    merged.diagnostics = diagnostics.diagnostics.clone();
                    if opts.format == ReportFormat::Text {
                        emit_validation(&opts, &diagnostics)?;
                        if !report.missing_mandatory.is_empty() {
                            println!(
                                "missingMandatory: {}",
                                report.missing_mandatory.join(", ")
                            );
                        }
                    } else {
                        emit_capability(&opts, &merged, false)?;
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
            format,
            out,
        } => {
            let opts = EmitOpts::from_flags(json, format.as_deref(), out)?;
            let baseline_contract = match parser::parse_file(&baseline) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let candidate_contract = match parser::parse_file(&candidate) {
                Ok(contract) => contract,
                Err(Error::InvalidDocument { report }) => {
                    emit_validation(&opts, &report)?;
                    return Ok(EXIT_FAILURE);
                }
                Err(err) => return Err(err),
            };
            let result = compare_contracts(&baseline_contract, &candidate_contract);
            let report = result.report();
            emit_compatibility(&opts, report)?;
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
        Commands::Registry {
            command: RegistryCommands::Serve { root, bind, token },
        } => {
            let addr: std::net::SocketAddr = bind.parse().map_err(|err| {
                Error::Serialization(format!("invalid --bind address `{bind}`: {err}"))
            })?;
            let options = ServeOptions {
                root,
                bind: addr,
                token,
            };
            let rt = tokio::runtime::Runtime::new().map_err(|err| {
                Error::Serialization(format!("failed to start async runtime: {err}"))
            })?;
            rt.block_on(async move { serve(options).await })
                .map_err(Error::Serialization)?;
            Ok(EXIT_OK)
        }
        Commands::Registry {
            command:
                RegistryCommands::Pull {
                    url,
                    token,
                    cache_dir,
                    json,
                },
        } => {
            let client = build_registry_client(&url, token, cache_dir)?;
            match client.get_registry() {
                Ok(registry) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&registry).map_err(|err| {
                            Error::Serialization(format!("serialize registry: {err}"))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("registry id: {}", registry.id);
                        println!("version: {}", registry.version);
                        println!("artifacts: {}", registry.artifacts.len());
                    }
                    Ok(EXIT_OK)
                }
                Err(err) => {
                    let code = exit_code_for_registry_error(&err);
                    let report = err.into_diagnostics();
                    print_report(&report, json)?;
                    Ok(code)
                }
            }
        }
        Commands::Registry {
            command:
                RegistryCommands::Lookup {
                    url,
                    id,
                    version,
                    token,
                    cache_dir,
                    json,
                },
        } => {
            let mut client = build_registry_client(&url, token, cache_dir)?;
            match client.lookup(&id, version.as_deref()) {
                Ok(artifact) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&artifact).map_err(|err| {
                            Error::Serialization(format!("serialize artifact: {err}"))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("id: {}", artifact.id);
                        println!("type: {}", artifact.artifact_type);
                        println!("version: {}", artifact.version);
                    }
                    Ok(EXIT_OK)
                }
                Err(err) => {
                    let code = exit_code_for_registry_error(&err);
                    let report = err.into_diagnostics();
                    print_report(&report, json)?;
                    Ok(code)
                }
            }
        }
        Commands::Registry {
            command:
                RegistryCommands::Publish {
                    url,
                    path,
                    content,
                    token,
                    cache_dir,
                    json,
                },
        } => {
            let raw = std::fs::read_to_string(&path).map_err(|err| Error::Io {
                path: path.clone(),
                source: err,
            })?;
            let mut artifact: crate::model::RegisteredArtifact =
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    serde_json::from_str(&raw)
                        .map_err(|err| Error::Serialization(err.to_string()))?
                } else {
                    serde_yaml::from_str(&raw)
                        .map_err(|err| Error::Serialization(err.to_string()))?
                };
            let id = artifact.id.clone();
            let content = match content {
                Some(content_path) => Some(std::fs::read_to_string(&content_path).map_err(
                    |err| Error::Io {
                        path: content_path,
                        source: err,
                    },
                )?),
                None => None,
            };
            if content.is_some() && artifact.location.is_none() {
                artifact.location = Some(format!(
                    "artifacts/{}-{}.yaml",
                    artifact.id, artifact.version
                ));
            }
            let request = PublishRequest {
                artifact,
                content,
                content_encoding: Some("utf-8".into()),
            };
            let client = build_registry_client(&url, token, cache_dir)?;
            match client.publish(&id, &request) {
                Ok(artifact) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&artifact).map_err(|err| {
                            Error::Serialization(format!("serialize artifact: {err}"))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("published: {}@{}", artifact.id, artifact.version);
                    }
                    Ok(EXIT_OK)
                }
                Err(err) => {
                    let code = exit_code_for_registry_error(&err);
                    let report = err.into_diagnostics();
                    print_report(&report, json)?;
                    Ok(code)
                }
            }
        }
        Commands::Registry {
            command:
                RegistryCommands::Deprecate {
                    url,
                    id,
                    version,
                    token,
                    cache_dir,
                    json,
                },
        } => {
            let client = build_registry_client(&url, token, cache_dir)?;
            match client.deprecate(&id, version.as_deref()) {
                Ok(artifact) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&artifact).map_err(|err| {
                            Error::Serialization(format!("serialize artifact: {err}"))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("deprecated: {}@{}", artifact.id, artifact.version);
                    }
                    Ok(EXIT_OK)
                }
                Err(err) => {
                    let code = exit_code_for_registry_error(&err);
                    let report = err.into_diagnostics();
                    print_report(&report, json)?;
                    Ok(code)
                }
            }
        }
        Commands::Registry {
            command:
                RegistryCommands::Retire {
                    url,
                    id,
                    version,
                    token,
                    cache_dir,
                    json,
                },
        } => {
            let client = build_registry_client(&url, token, cache_dir)?;
            match client.retire(&id, version.as_deref()) {
                Ok(artifact) => {
                    if json {
                        let payload = serde_json::to_string_pretty(&artifact).map_err(|err| {
                            Error::Serialization(format!("serialize artifact: {err}"))
                        })?;
                        println!("{payload}");
                    } else {
                        println!("retired: {}@{}", artifact.id, artifact.version);
                    }
                    Ok(EXIT_OK)
                }
                Err(err) => {
                    let code = exit_code_for_registry_error(&err);
                    let report = err.into_diagnostics();
                    print_report(&report, json)?;
                    Ok(code)
                }
            }
        }
        Commands::Registry {
            command: RegistryCommands::Cache { dir, clear },
        } => {
            let mut cache = RegistryCache::disk(&dir)?;
            if clear {
                cache.clear()?;
                println!("cache cleared: {}", dir.display());
            } else if let Some(root) = cache.root() {
                println!("cache dir: {}", root.display());
            }
            Ok(EXIT_OK)
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
        Commands::Package {
            command: PackageCommands::Validate { path, json },
        } => {
            let report = validate_package(&path);
            print_report(&report, json)?;
            Ok(exit_code_for_report(&report, false))
        }
        Commands::Package {
            command: PackageCommands::Show { path, json },
        } => {
            let layout = PackageLayout::open(&path)?;
            if json {
                let payload = serde_json::to_string_pretty(&layout.manifest).map_err(|err| {
                    Error::Serialization(format!("serialize package manifest: {err}"))
                })?;
                println!("{payload}");
            } else {
                println!("id: {}", layout.manifest.id);
                println!("version: {}", layout.manifest.version);
                println!("artifacts: {}", layout.manifest.artifacts.len());
            }
            Ok(EXIT_OK)
        }
        Commands::Package {
            command: PackageCommands::Pack { path, archive },
        } => {
            let layout = pack(&path, archive.as_deref())?;
            println!(
                "packed {}@{} ({} artifacts)",
                layout.manifest.id,
                layout.manifest.version,
                layout.manifest.artifacts.len()
            );
            if let Some(archive) = archive {
                println!("archive: {}", archive.display());
            }
            Ok(EXIT_OK)
        }
        Commands::Package {
            command: PackageCommands::Unpack { source, dest },
        } => {
            let layout = unpack(&source, &dest)?;
            println!(
                "unpacked {}@{} -> {}",
                layout.manifest.id,
                layout.manifest.version,
                dest.display()
            );
            Ok(EXIT_OK)
        }
        Commands::Schema {
            command: SchemaCommands::Json { out },
        } => {
            let written = write_document_schemas(&out)?;
            for path in written {
                println!("{path}");
            }
            Ok(EXIT_OK)
        }
        Commands::Schema {
            command: SchemaCommands::Openapi { kind, out },
        } => match kind.as_str() {
            "all" => {
                let written = write_openapi_documents(&out)?;
                for path in written {
                    println!("{path}");
                }
                Ok(EXIT_OK)
            }
            "documents" | "registry" => {
                let openapi_kind = if kind == "documents" {
                    OpenApiKind::Documents
                } else {
                    OpenApiKind::Registry
                };
                let doc = openapi_document(openapi_kind)?;
                let path = if out.extension().is_some() {
                    out
                } else {
                    std::fs::create_dir_all(&out).map_err(|err| Error::Io {
                        path: out.clone(),
                        source: err,
                    })?;
                    out.join(format!("{kind}.openapi.json"))
                };
                let body = serde_json::to_string_pretty(&doc)
                    .map_err(|err| Error::Serialization(format!("serialize openapi: {err}")))?;
                std::fs::write(&path, body).map_err(|err| Error::Io {
                    path: path.clone(),
                    source: err,
                })?;
                println!("{}", path.display());
                Ok(EXIT_OK)
            }
            other => Err(Error::Serialization(format!(
                "unknown schema openapi kind `{other}` (expected documents|registry|all)"
            ))),
        },
    }
}

fn print_report(report: &ValidationReport, json: bool) -> Result<(), Error> {
    let opts = EmitOpts::from_flags(json, None, None)?;
    emit_validation(&opts, report)
}

fn run_inspect_tui(path: &std::path::Path) -> Result<u8, Error> {
    #[cfg(feature = "tui")]
    {
        tui::run(path)
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = path;
        Err(Error::Serialization(
            "interactive inspector requires the `tui` feature (install `dpcs-cli` or build with `--features full`)"
                .to_owned(),
        ))
    }
}

fn build_registry_client(
    url: &str,
    token: Option<String>,
    cache_dir: Option<PathBuf>,
) -> Result<RegistryClient, Error> {
    let mut client =
        RegistryClient::new(url).map_err(|err| Error::Serialization(err.to_string()))?;
    if let Some(token) = token {
        client = client.with_token(token);
    }
    if let Some(dir) = cache_dir {
        client = client.with_cache(RegistryCache::disk(dir)?);
    }
    Ok(client)
}

fn exit_code_for_registry_error(err: &RegistryClientError) -> u8 {
    if err.is_transport() || err.is_server_error() {
        EXIT_FAILURE
    } else {
        EXIT_VALIDATION
    }
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
struct VersionPayload {
    version: String,
    dpcs_spec_version: String,
    conformance: crate::ConformanceClaim,
}
