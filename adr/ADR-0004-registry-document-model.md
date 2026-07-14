# ADR-0004: Registry document model in 0.9.0

## Status

Accepted

## Context

SPEC Ch 22 defines a Registry model. Network registry clients and
publication APIs are listed as future work (Appendix G) and appear on
ROADMAP 0.10.0 as “Registry client”.

## Decision

In 0.9.0, `dpcs` implements an in-process registry **document** COM
(`Registry`, `RegisteredArtifact`), `validate_registry`, and
`dpcs registry validate`. Operational discovery/publication over the
network remains deferred to 0.10.0.

## Consequences

- Registry conformance level means document parse/validate behavior.
- No HTTP/registry protocol dependencies in the crate.
