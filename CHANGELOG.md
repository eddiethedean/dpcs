# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.3.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.3.0
[0.2.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.2.0
[0.1.0]: https://github.com/eddiethedean/dpcs/releases/tag/v0.1.0
