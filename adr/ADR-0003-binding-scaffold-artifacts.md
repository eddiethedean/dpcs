# ADR-0003 — Binding Scaffold Artifacts

## Status

Accepted

## Context

SPEC Chapter 17 requires Orchestrator Binding to translate a validated Pipeline
Plan into platform-specific artifacts after capability matching. Artifact
formats are explicitly implementation-defined.

ROADMAP 0.8.0 requires adapters for Airflow, Dagster, Prefect, Temporal, and
Kubernetes.

## Decision

Ship a generic `bind` API that:

1. Evaluates a `CapabilityProfile` against the plan and fails closed on missing
   mandatory capabilities (retaining the structured capability report).
2. Dispatches to in-crate adapters that emit **idiomatic scaffold** files
   (Python DAG/flow/workflow stubs or Kubernetes Job/CronJob YAML).

Scaffolds preserve identity and declared topology with native edges/deps where
the platform allows. Independent steps never receive invented dependency edges.
Kubernetes linearizes `step_order` via initContainers (not parallel containers).
Quality gates, failure semantics, and execution requirements are documented in
headers/labels; they are not fully reified as platform runtimes.

Temporal and Kubernetes adapters are labeled experimental in CLI/docs but share
the same API surface.

Artifact writes reject path escape (`..` / absolute relative_path).

## Consequences

- Binding is available in-process and via `dpcs bind`.
- Execution runtimes remain out of scope.
- Future releases may deepen fidelity without changing the `BindingBundle`
  contract unless a compatibility break is intentional.
