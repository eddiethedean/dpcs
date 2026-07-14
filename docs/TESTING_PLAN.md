# Testing Plan

Required test categories:

## Parsing (0.3.0)

- parse valid YAML
- parse valid JSON
- reject malformed YAML (`DPCS-PARSE-001`, stage `parse`)
- reject malformed JSON (`DPCS-PARSE-001`)
- reject missing required fields (`DPCS-PARSE-002`)
- reject invalid field types (`DPCS-PARSE-002`)
- unsupported document format (`Error::UnsupportedFormat`)
- YAML document round-trip preserves COM
- JSON document round-trip preserves COM
- cross-format YAML → JSON → COM equivalence
- nested extension preservation through round-trip
- reserved colliding extensions omitted from wire serialization
- CLI validate/inspect/graph malformed document with `--json` includes `"stage": "parse"`

## Pipeline Graph (0.4.0)

- dependency graph traversal, predecessors/successors, transitive deps
- topological order stable across YAML key reorderings
- cycle path detection
- reject duplicate graph edges (`DPCS-GRP-005`)
- reject unreachable steps (`DPCS-GRP-006`)
- reject invalid entry/exit points (`DPCS-GRP-007`, `DPCS-GRP-008`)
- invalid data-flow ports do not create spurious graph dependencies
- cycles do not emit redundant unreachable-step diagnostics
- CLI `graph --json` includes entry/exit points and step order for valid contracts
- graph features round-trip (`entryPoints`, `exitPoints`, `dataFlow.contractRef`)

## Validation Engine (0.5.0)

- reject unresolved `transformRef` (`DPCS-REF-005`)
- reject unresolved step-port `contractRef` (`DPCS-REF-006`)
- reject missing data-flow `dataset` (`DPCS-DF-004`)
- reject unreachable datasets (`DPCS-DF-005`)
- reject unsatisfied step inputs / interface outputs (`DPCS-DF-006`)
- reject illegal data-flow source/destination roles (`DPCS-DF-007`, `DPCS-DF-008`)
- reject conflicting control vs graph/data deps (`DPCS-CF-004`)
- reject duplicate control-flow edges (`DPCS-CF-005`)
- reject empty step port ids (`DPCS-STR-001`)
- accept same graph endpoints with different `kind` values
- cycles do not suppress unreachable-dataset diagnostics
- invalid entry points do not flood unreachable-step diagnostics
- graph features fixture validates with interface-rooted data flow

## Execution Model (0.6.0)

- validate execution capabilities, isolation, external deps, resources, environment (`DPCS-EXE-*`)
- validate scheduling modes, events, and ISO-8601 timing constraints (`DPCS-SCH-*`)
- validate quality gate purpose/criteria/outcomes/placement (`DPCS-QG-*`)
- validate failure scope/triggers/responses/retry (`DPCS-FS-*`)
- validate lineage datasets/steps/refs and reject legacy upstream/downstream stubs (`DPCS-LIN-*`)
- `plan()` refuses invalid contracts with `DPCS-PLN-001`
- independent steps use deterministic sorted-id topo tie-break
- CLI `inspect`/`graph` omit fake `stepOrder` and signal `planningRefused`
- execution-model YAML/JSON round-trip
- validate shipped example: `examples/with_execution.dpcs.yaml`

## Capability Model (0.7.0)

- validate profiles for empty/duplicate capability ids and identity/version
- `evaluate` matches plan demand (`requiredCapabilities`, external dependency capabilities)
- environment `softwareCapabilities` / `isolation` are not orchestrator demand
- missing mandatories emit `DPCS-CAP-005` with sourced `object_ref`; version mismatch warns `DPCS-CAP-006`
- `evaluate_requirements` checks without planning; `evaluate_many` ranks profiles (ok, fewer missing, more satisfied)
- failure results retain structured `CapabilityReport.missing_mandatory`
- CLI `capabilities` exit codes and `--json` report shape
- match shipped example pair: `examples/orchestrator.capabilities.yaml` + `examples/with_execution.dpcs.yaml`

## Orchestrator Binding (0.8.0)

- `BindingFramework::is_available()` and five `supported_targets`
- `bind` succeeds for Airflow/Dagster/Prefect/Temporal/Kubernetes after capability match
- scaffolds contain contract id and step ids; multi-step plans encode dependencies
- Kubernetes emits CronJob when scheduling has cron
- missing mandatory capabilities refuse bind with `DPCS-BIND-001` (+ CAP findings)
- `bind_contract` refuses invalid contracts via planning diagnostics
- `write_bundle` materializes relative paths
- CLI `bind` success, `--json` bundle shape, capability refusal exit 1, unknown target `DPCS-BIND-002`

## Validation

- reject duplicate step identifiers (`DPCS-COM-005`)
- reject duplicate interface ports across inputs/outputs (`DPCS-COM-013`)
- reject incomplete interface ports (`DPCS-COM-006`–`DPCS-COM-011`)
- reject invalid graph edges
- reject prohibited cycles
- reject unresolved contract references (including bare filenames and data-flow refs)
- validate data flow endpoints (including declared step ports)
- validate control flow dependencies
- preserve root extension fields
- reserved extension key collision on constructed maps (`DPCS-COM-012`)
- deterministic diagnostics

## Canonical Object Model (0.2.0)

- pipeline identity extraction
- identity catalog lookup by path and kind
- YAML and JSON produce equal COM values for equivalent documents
- `ExtensionValue` round-trip through `serde_json::Value`

## CLI

- CLI success/failure exit codes (`0` / `1` / `2`)
- `inspect --json` summary shape including planning status
- `diagnostics --json` includes diagnostic ids
- `graph --json` omits `stepOrder` when planning is refused
- `capabilities` match success and missing-mandatory failure
- `bind` writes artifacts, `--json` emits `BindingBundle`, capability/target failures exit 1
- validate shipped example: `examples/minimal.dpcs.yaml`
