# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-07-14

### Added

- Cargo workspace: `dpcs` library crate and `dpcs-cli` binary crate
- JSON Schema generation (`jsonschema` feature, `dpcs schema json`) and OpenAPI
  helpers (`dpcs schema openapi`) with committed `schemas/` artifacts
- Pipeline package format (`.dpcspkg`): `PackageManifest`, pack/unpack/validate/resolve,
  CLI `dpcs package`, example `examples/packages/minimal.dpcspkg`
- Reference registry HTTP API (ADR-0005): file-backed `dpcs registry serve`,
  `RegistryClient` + cache, CLI pull/lookup/publish/deprecate/retire/cache,
  diagnostics `DPCS-REGC-*`
- Python bindings (`bindings/python`, maturin / PyO3) with PyPI release job
- WASM bindings (`bindings/wasm`, wasm-bindgen) with npm and Wasmer release jobs
- Docs: `docs/PACKAGE_FORMAT.md`, `docs/REGISTRY_API.md`

### Changed

- Crate version `0.10.0`; default library features are empty (CLI enables `full`)
- Release workflow publishes crates.io, PyPI, npm, and Wasmer
- Raise MSRV to 1.86 for locked `url`/`idna`/`icu_*` dependency builds

## [0.9.0] - 2026-07-14

### Added

- Diagnostics completeness (SPEC Ch 18): `relatedDiagnostics`, `DiagnosticReport`,
  `ProcessingResult`, `ImplementationMetadata`, `validate_diagnostic`,
  `Severity::Information` helpers, `DiagnosticStage::CompatibilityAnalysis`
- SemVer-compatible versioning helpers (`parse_version`, `is_valid_version`,
  `versions_compatible`) with `DPCS-VER-001`–`003`
- Optional independent `PipelinePlan.version`
- Extensibility validation (`DPCS-EXT-001`–`010`, `ExtensionDefinition`,
  namespace rules for `x-*` / `vendor:name` / URI-like keys)
- Compatibility analysis: `compare_contracts`, `compare_plans`,
  `CompatibilityCategory` / `CompatibilityReport` / `CompatibilityResult`,
  diagnostics `DPCS-COMPAT-*`, CLI `dpcs compatibility`
- Security metadata COM + validation (`DPCS-SEC-001`–`005`)
- Governance metadata COM + validation (`DPCS-GOV-001`–`005`)
- Registry document model (`Registry`, `RegisteredArtifact`, `validate_registry`,
  `DPCS-REG-001`–`014`), CLI `dpcs registry validate` (no network client; ADR-0004)
- Conformance profiles/claims (`ConformanceProfile`, `ConformanceClaim`,
  `ConformanceLevel`, `DPCS-CONF-*`), CLI `dpcs conformance validate`,
  `dpcs version --json` emits toolkit claim
- Appendix E–aligned suite in `tests/conformance.rs`
- Examples: compatibility pairs, `registry.yaml`, `conformance.profile.yaml`,
  `with_security_governance.dpcs.yaml`

### Changed

- Crate version `0.9.0`; architecture documents Chapters 18–25
- Extension validation phase is no longer a no-op
- `dpcs version` prints spec version and conformance levels

### Fixed

- Capability profile / evaluate reuse shared `versions_compatible` helper
- Compatibility now diffs step ports / `transformRef`, control-flow `kind`, and
  interface `name`/`purpose`; classifies additive plan changes as backwardCompatible;
  compares execution/scheduling/quality/failure via order-insensitive fingerprints
- `DPCS-SEC-003` no longer rejects non-empty non-secret annotation values
- Renamed nonstandard compatibility-mode warning to `DPCS-COMPAT-050` (was colliding with
  contract-reference removal `DPCS-COMPAT-010`)
- Conformance profile validation checks toolkit version compatibility and claimed levels
- `dpcs validate --profile` applies conformance constraints; validate/diagnostics `--json`
  emit `DiagnosticReport`
- `DPCS-PLN-001` links related validation error ids; empty registry `publicationStatus`
  is an error (`DPCS-REG-015`)

## [0.8.0] - 2026-07-14

### Added

- Orchestrator binding framework (SPEC Ch 17): `bind`, `bind_contract`, `parse_target`, `write_bundle`
- `BindingTarget`, `BindingFile`, `BindingBundle`, `BindingResult`, `BindContext`, and `BindingFramework::supported_targets()`
- Scaffold adapters for Airflow, Dagster, Prefect, Temporal (experimental), and Kubernetes (experimental)
- Capability-gated binding: missing mandatory capabilities refuse bind with `DPCS-BIND-001`
- Binding diagnostics `DPCS-BIND-001`–`004`, category `binding`, stage `OrchestratorBinding`
- CLI `dpcs bind <contract> --profile <profile> --target <…> [--out <dir>] [--json]`
- Binding integration and CLI tests

### Changed

- `BindingFramework::is_available()` returns `true`
- Architecture and docs extend through Orchestrator Binding; execution runtimes remain out of scope

### Fixed

- Airflow no longer invents linear `>>` edges when the plan has no dependency edges
- Dagster / Prefect adapters wire declared dependencies natively (`upstream` / `wait_for`)
- Kubernetes runs steps via `initContainers` + main container (not parallel peer containers)
- `write_bundle` rejects path escape (`..` / absolute) with `DPCS-BIND-004`
- Sanitized identifiers are disambiguated on collision; Temporal workflow classes use PascalCase
- Capability-gate refusals retain structured `CapabilityReport` on `BindingResult::Err`
- CronJob scaffolds emit `timeZone` when SchedulingIntent provides a timezone

## [0.7.0] - 2026-07-14

### Added

- `CapabilityProfile` / `CapabilityDecl` COM with YAML/JSON load helpers and serde alias `profile` → `identity`
- Profile validation (`validate_profile`) with `DPCS-CAP-001`–`004`
- Post-plan capability matching: `evaluate`, `evaluate_requirements`, `evaluate_many`
- `CapabilityReport` / `CapabilityResult` plus `DPCS-CAP-005` (missing mandatory) and `DPCS-CAP-006` (version mismatch warning)
- Diagnostic category `capability` and stage helpers for `CapabilityEvaluation`
- CLI `dpcs capabilities <profile> --plan <contract> [--json]`
- Example `examples/orchestrator.capabilities.yaml` and capability fixtures/tests
- `PipelinePlan.dpcs_version` carried from the source contract for profile compatibility checks

### Changed

- Orchestrator capability matching is no longer deferred; binding adapters remain ROADMAP 0.8.0
- `OrchestratorCapabilities` is a deprecated name alias for `CapabilityProfile`
- Capability demand for matching is `requiredCapabilities` + `externalDependencies[].capability` (environment software/isolation are not orchestrator ids)
- `CapabilityResult::Err` retains a structured `CapabilityReport` alongside diagnostics
- `evaluate_many` ranks profiles (successful matches first, then fewer missing, then more satisfied)
- Capability profile wire format accepts bare capability id strings; omitted `dpcsVersion` defaults to empty (then `DPCS-CAP-004`)

### Fixed

- `DPCS-CAP-005` `object_ref` now cites the demand source (`requiredCapabilities` vs `externalDependencies.capability`)
- Failure evaluations expose `missing_mandatory` on the retained capability report
- CAP-006 compares profile version against both toolkit and plan/contract `dpcsVersion` when available
- Topological `stepOrder` uses a sorted ready-set tie-break across independent dependency chains
- `evaluate_many` ranks invalid profiles after incomplete but valid matches
- Scheduling timing comparisons (`DPCS-SCH-006`) require matching timezone offsets; mixed offsets warn via `DPCS-SCH-007`

## [0.6.0] - 2026-07-14

### Added

- Structured `ExecutionRequirements` with resources, environment, isolation, and external dependencies (`DPCS-EXE-001`–`006`)
- Structured `SchedulingIntent` list with modes, events, and constraints (`DPCS-SCH-001`–`007`)
- Quality gate criteria, outcomes, placement, and legacy-field rejection (`DPCS-QG-001`–`009`)
- Failure semantics scope, triggers, responses, retry, and legacy-field rejection (`DPCS-FS-001`–`008`)
- Pipeline lineage dataset/step/provenance validation (`DPCS-LIN-001`–`016`)
- Full `PipelinePlan` IR with gated deterministic planning (`DPCS-PLN-001`)
- Public re-exports for execution-model COM types, `plan` / `try_plan`, and `PlanResult`
- Example `examples/with_execution.dpcs.yaml` and fixtures for execution-model validation

### Changed

- `scheduling` is now `Vec<SchedulingIntent>` (was `Option<SchedulingIntent>`)
- `QualityGate` requires `purpose`, `criteria`, `onSuccess`, and `onFailure` (replaced free-form `scope`/`rule`)
- `FailureSemantics` requires structured `scope`, `triggers`, and `responses` (replaced free-form `onFailure`)
- `PipelineLineage` uses dataset/step/provenance structures (replaced loose `upstream`/`downstream` strings)
- `plan()` returns `PlanResult` and refuses invalid contracts
- CLI `inspect` / `graph` surface `planningRefused` and omit fake `stepOrder` when planning fails
### Fixed

- Retry responses require meaningful `retry` policy (reject empty `retry: {}` and `eligible: false`)
- Timing constraint checks only compare RFC3339/ISO-8601 timestamps; free-form times warn via `DPCS-SCH-007`
- Lineage always warns when declared datasets are absent from `dataFlow` (`DPCS-LIN-002`)
- Legacy stub keys (`lineage.upstream`/`downstream`, QG `scope`/`rule`, FS `onFailure`) are rejected instead of silently absorbed
- Empty quality outcomes and failure responses are rejected
- Duplicate identity detection treats trimmed ids as equal (`DPCS-COM-005`)
- Scheduling mode deserialization accepts common casing variants of known modes

## [0.5.0] - 2026-07-13

### Added

- Reference validation for `transformRef` (`DPCS-REF-005`) and step-port `contractRef` (`DPCS-REF-006`)
- Data-flow dataset identity (`DPCS-DF-004`), unreachable datasets (`DPCS-DF-005`), unsatisfied ports (`DPCS-DF-006`), and endpoint roles (`DPCS-DF-007` / `DPCS-DF-008`)
- Control-flow conflict detection against opposite graph/data-flow deps (`DPCS-CF-004`)
- Duplicate control-flow edge detection (`DPCS-CF-005`)
- Structural empty step-port id check (`DPCS-STR-001`)
- Dataset reachability helpers `unreachable_datasets` / `unsatisfied_ports`
- Fixtures and integration tests for the new validation rules

### Changed

- `DPCS-GRP-005` messages include edge `kind` (or `untyped`)
- Quality, failure, and extension validation phases remain intentional stubs until ROADMAP 0.6 / 0.9

### Fixed

- Dataset reachability no longer vacuously sources portless steps; requires interface-rooted provenance
- Graph cycles no longer suppress unreachable-dataset diagnostics (`DPCS-DF-005`)
- Data-flow endpoint role checks (`DPCS-DF-007` / `DPCS-DF-008`) enforce SPEC source/destination rules
- Invalid `entryPoints` no longer flood `DPCS-GRP-006` for every step
- Control-flow conflicts (`DPCS-CF-004`) ignore already-bidirectional graph pairs covered by cycle detection

## [0.4.0] - 2026-07-13

### Added

- `DependencyGraph` analysis APIs: traversal, transitive dependencies/dependents, cycle detection, topological ordering, unreachable steps, duplicate edge detection
- `PipelineGraph` fields: `entryPoints`, `exitPoints`, `metadata`
- `DataFlow::contract_ref` for associated contract references
- `PipelineStep::input_port` / `output_port`, `ContractReference::matches_id`, `PipelineContract::step_ids`
- Graph diagnostics `DPCS-GRP-005`–`DPCS-GRP-008` (duplicate edges, unreachable steps, invalid entry/exit points)
- Shared data-flow endpoint resolution for validation and dependency analysis
- Re-export graph analysis and COM types from crate root (`DependencyGraph`, `PipelineGraph`, `PipelineStep`, …)
- Fixtures and integration tests for graph analysis and new validation rules

### Changed

- Graph validation uses `DependencyGraph` for cycle and reachability analysis
- `plan::plan` uses topological ordering when the dependency graph is acyclic
- CLI `graph` includes `entryPoints` and `exitPoints` in text and JSON output
- `DPCS_SPEC_VERSION` aligned with the draft specification label (`1.0.0-draft`)

### Fixed

- Data-flow graph analysis only adds inter-step edges for validated endpoints, avoiding false cycles and missed unreachable steps
- Invalid `graph.entryPoints` / `graph.exitPoints` are reported (`DPCS-GRP-007`, `DPCS-GRP-008`) instead of being silently ignored
- `plan::plan` preserves declaration order for steps omitted from topological ordering
- `dataFlow[].contractRef` is validated against `contractReferences`
- Bare filename `contractRef` values (for example `bogus.yaml`) are no longer treated as direct paths
- `DPCS-GRP-006` is suppressed when a cycle is already reported; unreachable messaging distinguishes entry points from roots
- Cycle diagnostics reference graph, control flow, and data flow sources
- `find_cycle` returns a minimal cycle path

## [0.3.0] - 2026-07-13

### Added

- Parse-stage diagnostics (`DPCS-PARSE-001`, `DPCS-PARSE-002`) with `syntax` category and optional `sourceLocation`
- `Error::InvalidDocument` carrying a `ValidationReport` for invalid YAML/JSON documents
- Serialization APIs: `to_yaml`, `to_json`, `to_yaml_file`, `to_json_file`, `to_file`
- `PipelineContract::to_yaml_str`, `to_json_str`, and matching file writers
- Document round-trip and nested extension preservation tests

### Changed

- YAML/JSON document failures use `Error::InvalidDocument` (Parse-stage `ValidationReport`) instead of raw `Error::Yaml` / `Error::Json`
- CLI `validate` and `diagnostics` print parse reports (text or `--json`) and exit with code `2`
- Re-export `parse_file` and serialize helpers from the crate root
- Schema/type deserialize failures map to `DPCS-PARSE-002`; syntax failures remain `DPCS-PARSE-001`
- File parse failures include the document path in `sourceLocation`
- Wire serialization omits reserved colliding root extension keys to avoid duplicate keys
- Format-neutral `ExtensionValue` deserialization preserves YAML nulls and non-finite numbers

### Fixed

- CLI `inspect`/`graph` honor `--json` for parse-stage failures
- `DPCS-COM-013` only reports cross-side interface port collisions (same-side duplicates stay `DPCS-COM-005`)
- `Error::InvalidDocument` Display includes diagnostic id and message
- Missing-field parse remediation names the absent field when available

## [0.2.0] - 2026-07-13

### Added

- First-class identity model: `ObjectId`, `PipelineIdentity`, `ObjectKind`, `ObjectPath`, `IdentityCatalog`
- `PipelineContract::identity()` and `PipelineContract::identity_catalog()`
- Format-neutral `ExtensionValue` and `ExtensionMap` for COM extension storage
- COM invariant validation phase (`canonicalObjectModel` diagnostics)
- Interface metadata slot and port helper methods on `PipelineInterface`
- `InterfacePort::is_complete()` for SPEC Ch 4 port completeness checks
- Cross-side interface port uniqueness diagnostic (`DPCS-COM-013`)
- Model-focused tests for identity, COM invariants, and cross-format equivalence

### Changed

- COM extension fields now use `ExtensionMap` instead of `serde_json::Value`
- Reserved extension key collision checks moved to the COM validation phase
- Identifier presence/uniqueness owned by COM; document/structural phases no longer duplicate those errors
- Data-flow endpoint matching requires declared step ports when present
- Valid examples and fixtures populate required interface port properties (`name`, `purpose`)

### Fixed

- Duplicate DOC/STR diagnostics for the same identity faults
- Ambiguous `DPCS-REF-004` object references missing `inputs`/`outputs`
- Empty-id catalog paths colliding on trailing dots
- CLI JSON serialize failures now exit as I/O/failure (code 2) instead of silently continuing
- Over-permissive data-flow acceptance of arbitrary `steps.<id>.inputs|outputs…` suffixes
- Raise MSRV to 1.85 so locked clap dependencies (edition 2024) build in CI

## [0.1.0] - 2026-07-03

### Added

- Project foundation for the DPCS reference implementation
- Canonical Object Model aligned with `SPEC.md`
- YAML and JSON parsers with extension field preservation
- Phase-based validation and deterministic diagnostics
- Pipeline Plan and capability-model skeletons
- `dpcs` CLI: `validate`, `inspect`, `diagnostics`, `graph`, `version`
- Examples, fixtures, CI, and contributor documentation

[0.10.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.10.0
[0.9.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.9.0
[0.8.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.8.0
[0.7.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.7.0
[0.6.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.6.0
[0.5.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.5.0
[0.4.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.4.0
[0.3.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.3.0
[0.2.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.2.0
[0.1.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.1.0
