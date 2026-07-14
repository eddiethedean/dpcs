# CLI Specification

Binary name:

```bash
dpcs
```

Commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs capabilities <profile> --plan <contract>
dpcs bind <contract> --profile <profile> --target <airflow|dagster|prefect|temporal|kubernetes>
dpcs compatibility <baseline> <candidate>
dpcs registry validate <path>
dpcs conformance validate <path>
dpcs version [--json]
```

Output modes:

```bash
dpcs validate pipeline.yaml
dpcs validate pipeline.yaml --json
dpcs validate pipeline.yaml --strict
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml --json
dpcs bind pipeline.yaml --profile orchestrator.capabilities.yaml --target airflow
dpcs bind pipeline.yaml --profile orchestrator.capabilities.yaml --target airflow --out ./out --json
dpcs compatibility baseline.yaml candidate.yaml --json
dpcs registry validate registry.yaml --json
dpcs conformance validate conformance.profile.yaml --json
dpcs version --json
```

Exit codes:

- `0` valid / capability match ok / bind ok / compatible / registry or conformance ok
- `1` validation, capability, binding, compatibility, registry, or conformance errors
- `2` parse or IO failure

Notes:

- `validate` and `diagnostics` use the exit codes above.
- `capabilities` parses the contract, runs `plan()`, then `evaluate` against the profile. Exit `0` on match, `1` on plan refusal or capability errors, `2` on parse/I/O.
- `capabilities --json` emits a `CapabilityReport` on both success and match failure (failure includes diagnostics and `missingMandatory`).
- `bind` parses the contract and profile, plans, capability-gates, then translates to scaffold artifacts. Writes under `--out` (default `./dpcs-bind-<target>/`). Exit `0` on success, `1` on plan/capability/binding errors (including unknown `--target`), `2` on parse/I/O or write failure.
- `bind --json` emits a `BindingBundle` on success (and still writes files when `--out` is used or defaulted).
- `--target` accepts `airflow`, `dagster`, `prefect`, `temporal`, `kubernetes`, and alias `k8s`.
- `temporal` and `kubernetes` targets are experimental.
- `compatibility` compares two Pipeline Contracts. Exit `0` when the category is compatible (fully / backward / forward / conditional), `1` when incompatible, `2` on parse/I/O. `--json` emits `CompatibilityReport`.
- `registry validate` validates an in-process registry document (`DPCS-REG-*`). Network registry clients are out of scope (ROADMAP 0.10).
- `conformance validate` validates a conformance profile and checks claimed levels against this toolkit.
- `version --json` emits toolkit version, `dpcsSpecVersion`, and the toolkit `ConformanceClaim`.
- Invalid YAML/JSON documents emit Parse-stage diagnostics (`DPCS-PARSE-*`) and exit `2`. With `--json`, the diagnostic report is printed to stdout for `validate`, `diagnostics`, `inspect`, `graph`, `capabilities`, `bind`, `compatibility`, `registry`, and `conformance` failures.
- `inspect` and `graph` always exit `0` after a successful parse and do not fail on validation errors.
- `inspect` reports `valid`, execution-model counts, and `planningRefused` / `stepOrder` when a plan is available.
- `graph` prints topology always; planned `stepOrder` is omitted when planning is refused, and both text/JSON report `planningRefused`.
- JSON serialization failures for reports exit with code `2`.
