# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-13

### Added

- First-class identity model: `ObjectId`, `PipelineIdentity`, `ObjectKind`, `ObjectPath`, `IdentityCatalog`
- `PipelineContract::identity()` and `PipelineContract::identity_catalog()`
- Format-neutral `ExtensionValue` and `ExtensionMap` for COM extension storage
- COM invariant validation phase (`canonicalObjectModel` diagnostics)
- Interface metadata slot and port helper methods on `PipelineInterface`
- `InterfacePort::is_complete()` for SPEC Ch 4 port completeness checks
- Model-focused tests for identity, COM invariants, and cross-format equivalence

### Changed

- COM extension fields now use `ExtensionMap` instead of `serde_json::Value`
- Reserved extension key collision checks moved to the COM validation phase
- Valid examples and fixtures populate required interface port properties (`name`, `purpose`)

## [0.1.0] - 2026-07-03

### Added

- Project foundation for the DPCS reference implementation
- Canonical Object Model aligned with `SPEC.md`
- YAML and JSON parsers with extension field preservation
- Phase-based validation and deterministic diagnostics
- Pipeline Plan and capability-model skeletons
- `dpcs` CLI: `validate`, `inspect`, `diagnostics`, `graph`, `version`
- Examples, fixtures, CI, and contributor documentation
