# ADR-0001 — Rust-First Implementation

## Status

Accepted

## Context

DPCS needs a reliable, fast, embeddable reference implementation.

## Decision

The first implementation SHALL be a Rust crate.

## Consequences

Rust will provide:

- CLI
- validation core
- future Python bindings
- future WASM bindings
- future orchestrator compilers
