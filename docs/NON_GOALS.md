# Non-Goals Beyond 0.13.0

Completed through 0.13.0: full toolkit-actionable SPEC coverage (see
[`SPEC_COVERAGE.md`](SPEC_COVERAGE.md)), Appendix E conformance suite, public API
freeze, contract reference resolution, nested pipelines, structured bind
semantics, orchestrator binding scaffolds, SPEC Ch 18–25 document models, JSON
Schema/OpenAPI helpers, reference registry HTTP API, pipeline packages,
Python/WASM distribution packages, Markdown/HTML reports, Mermaid/DOT graph
exports, rich CLI `--format`/`--out`, the interactive TUI inspector,
parallel/incremental validation, shared analysis context, Criterion benches, and
practical allocation reductions (see [`BINDINGS.md`](BINDINGS.md),
[`CLI_SPEC.md`](CLI_SPEC.md), [`VALIDATION_GUIDE.md`](VALIDATION_GUIDE.md)).

Do **not** implement yet:

- production-complete Airflow operators / Dagster resources / Temporal workers
- execution runtime
- scheduling runtime
- actual data movement
- ETL execution
- ODCS validation internals
- DTCS validation internals
- lifetime-parameterized / wire-buffer-borrowing COM (`PipelineContract<'a>`)
- multi-document TUI workspace browser / registry-serve TUI

DPCS should reference ODCS and DTCS contracts, not implement them internally.
Binding produces orchestration artifacts; it does not run pipelines.
The reference registry HTTP API is toolkit-local (ADR-0005), not the sole future SPEC protocol.
