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
- plan preserves declaration-order steps omitted from topological order
- CLI `graph --json` includes entry/exit points and step order
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
- `inspect --json` summary shape
- `diagnostics --json` includes diagnostic ids
- validate shipped example: `examples/minimal.dpcs.yaml`
