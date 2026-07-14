# DPCS ROADMAP

**Project:** `dpcs`\
**Language:** Rust\
**Goal:** Build the reference implementation of the Data Pipeline
Contract Standard (DPCS).

## Vision

`dpcs` will become the canonical Rust implementation of DPCS and the
foundation for Python, WebAssembly, and other language bindings.

Implementation proceeds in incremental 0.x releases, with each release
completing a coherent portion of the specification while maintaining a
stable public API where practical.

------------------------------------------------------------------------

# 0.1.0 --- Project Foundation (shipped)

## Goals

-   Workspace and crate structure
-   CI/CD
-   Formatting and linting
-   Documentation
-   CLI skeleton

### Deliverables

-   Cargo workspace
-   Module layout
-   GitHub Actions
-   `dpcs validate`
-   `dpcs version`
-   Test harness

------------------------------------------------------------------------

# 0.2.0 --- Canonical Object Model (shipped)

Implements Chapters 1--4.

### Deliverables

-   PipelineContract
-   PipelineInterface
-   Metadata
-   Identity model
-   Serialization-independent COM

------------------------------------------------------------------------

# 0.3.0 --- Parsing (shipped)

### Deliverables

-   YAML parser
-   JSON parser
-   Extension preservation
-   Parse diagnostics
-   Round-trip serialization tests

------------------------------------------------------------------------

# 0.4.0 --- Pipeline Graph (shipped)

Implements Chapters 5--9.

### Deliverables

-   PipelineGraph
-   PipelineStep
-   ContractReference
-   DataFlow
-   ControlFlow
-   Graph traversal
-   Cycle detection
-   Dependency analysis

------------------------------------------------------------------------

# 0.5.0 --- Validation Engine (shipped)

### Deliverables

Phase-based validation:

1.  Document
2.  COM
3.  Structure
4.  Graph
5.  References
6.  Data Flow
7.  Control Flow
8.  Extensions

Rich diagnostics.

------------------------------------------------------------------------

# 0.6.0 --- Execution Model (shipped)

Implements Chapters 10--15.

### Deliverables

-   ExecutionRequirements
-   SchedulingIntent
-   QualityGate
-   FailureSemantics
-   PipelineLineage
-   PipelinePlan
-   Deterministic planning

------------------------------------------------------------------------

# 0.7.0 --- Capability Model (shipped)

Implements Chapter 16.

### Deliverables

-   Capability profiles
-   Capability matcher
-   Capability validation
-   Capability reports

------------------------------------------------------------------------

# 0.8.0 --- Orchestrator Binding (shipped)

Implements Chapter 17.

### Deliverables

Generic binding framework.

Initial adapters:

-   Airflow
-   Dagster
-   Prefect

Experimental:

-   Temporal
-   Kubernetes

------------------------------------------------------------------------

# 0.9.0 --- Complete Specification (shipped)

Implements Chapters 18--25 (Chapter 26 appendices are informative).

### Deliverables

-   Diagnostics (Ch 18 completeness: related diags, DiagnosticReport, stages)
-   Compatibility analysis (`compare_contracts` / `compare_plans`)
-   Versioning (SemVer-compatible syntax validation)
-   Extensibility (namespace rules + ExtensionDefinition)
-   Registries (in-process document model; see ADR-0004)
-   Conformance (claims/profiles + `tests/conformance`)
-   Security metadata
-   Governance metadata

Full conformance suite (Appendix E checklist).

------------------------------------------------------------------------

# 0.10.0 --- Ecosystem (shipped)

### Deliverables

-   Python bindings (PyO3 / maturin → PyPI `dpcs`)
-   WASM bindings (wasm-bindgen → npm `@eddiethedean/dpcs` + Wasmer `eddiethedean/dpcs`)
-   JSON Schema generation (`dpcs schema json`)
-   OpenAPI helpers (`dpcs schema openapi`)
-   Registry client + reference HTTP server (`dpcs registry serve` / pull / lookup / publish)
-   Package support (`.dpcspkg` + `dpcs package`)

See [`docs/BINDINGS.md`](docs/BINDINGS.md) for install and republish details.

------------------------------------------------------------------------

# 0.11.0 --- Developer Experience (shipped)

### Deliverables

-   TUI inspector (`dpcs tui` / `dpcs inspect --tui`, feature `tui`)
-   Graph visualization (Mermaid + Graphviz DOT + HTML Mermaid page)
-   Rich CLI (`--format` / `--out`, colored text; `--json` alias)
-   HTML reports
-   Markdown reports
-   Library `report` module (`InspectView`, `GraphView`, renderers)

------------------------------------------------------------------------

# 0.12.0 --- Performance (shipped)

### Deliverables

-   Zero-copy parsing where practical (owned COM; borrow-heavy analysis + clone-free wire serialize)
-   Parallel validation (`parallel` feature / CLI `full`)
-   Incremental validation (`ValidationCache` / `validate_cached`)
-   Large graph optimization (`AnalysisContext`, indexed endpoints, graph reuse)
-   Benchmark suite (`cargo bench -p dpcs` / `make bench`)

------------------------------------------------------------------------

# 0.13.0 --- Reference Implementation

### Deliverables

-   Complete implementation of SPEC.md
-   100% conformance suite
-   Stable public API
-   Comprehensive documentation

------------------------------------------------------------------------

# 1.0.0

## Goals

-   Entire DPCS specification implemented
-   Stable API
-   Stable file formats
-   Registry interoperability
-   Production-ready CLI
-   Complete examples
-   Long-term support commitment

## Success Criteria

-   Full SPEC.md coverage
-   Complete conformance tests
-   Rust reference implementation
-   Python bindings available
-   WASM bindings available
-   High-quality documentation
-   Ready for ecosystem adoption
