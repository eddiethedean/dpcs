# CLI Specification

Binary name:

```bash
dpcs
```

Initial commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs capabilities <profile> --plan <contract>
dpcs version
```

Output modes:

```bash
dpcs validate pipeline.yaml
dpcs validate pipeline.yaml --json
dpcs validate pipeline.yaml --strict
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml --json
```

Exit codes:

- `0` valid / capability match ok
- `1` validation or capability errors
- `2` parse or IO failure

Notes:

- `validate` and `diagnostics` use the exit codes above.
- `capabilities` parses the contract, runs `plan()`, then `evaluate` against the profile. Exit `0` on match, `1` on plan refusal or capability errors, `2` on parse/I/O.
- `capabilities --json` emits a `CapabilityReport` on both success and match failure (failure includes diagnostics and `missingMandatory`).
- Invalid YAML/JSON documents emit Parse-stage diagnostics (`DPCS-PARSE-*`) and exit `2`. With `--json`, the diagnostic report is printed to stdout for `validate`, `diagnostics`, `inspect`, `graph`, and `capabilities` failures.
- `inspect` and `graph` always exit `0` after a successful parse and do not fail on validation errors.
- `inspect` reports `valid`, execution-model counts, and `planningRefused` / `stepOrder` when a plan is available.
- `graph` prints topology always; planned `stepOrder` is omitted when planning is refused, and both text/JSON report `planningRefused`.
- JSON serialization failures for reports exit with code `2`.
