# ADR-0005: Reference registry HTTP API in 0.10.0

## Status

Accepted

## Context

SPEC Chapter 22 defines registry documents and validation but leaves discovery,
lookup, publication, and related operations implementation-defined. Appendix G
lists “Standardized Pipeline Registry APIs” as future work. ADR-0004 deferred
network clients from 0.9.0.

## Decision

In 0.10.0, `dpcs` ships:

1. A **reference HTTP API** documented in `docs/REGISTRY_API.md` and
   `schemas/registry.openapi.json`.
2. A **file-backed reference server** (`dpcs registry serve`).
3. A **Rust client** with optional disk cache (`RegistryClient`).

This protocol is normative for the DPCS reference toolkit only. It does not claim
to be the only future standardized registry API.

## Consequences

- Registry conformance remains rooted in Ch 22 document semantics.
- `reqwest` / `axum` are optional features (`registry-client` / `registry-server`).
- Alternative registry transports may coexist without changing the COM.
