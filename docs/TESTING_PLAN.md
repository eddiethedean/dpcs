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

## Validation

- reject duplicate step identifiers (`DPCS-COM-005`)
- reject duplicate interface ports across inputs/outputs (`DPCS-COM-013`)
- reject incomplete interface ports (`DPCS-COM-006`–`DPCS-COM-011`)
- reject invalid graph edges
- reject prohibited cycles
- reject unresolved contract references
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
