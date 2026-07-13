# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.5.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.5.0
[0.4.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.4.0
[0.3.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.3.0
[0.2.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.2.0
[0.1.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.1.0
