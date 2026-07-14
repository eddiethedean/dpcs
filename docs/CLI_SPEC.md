# CLI Specification

Binary name:

```bash
dpcs
```

Commands:

```bash
dpcs validate <path> [--json] [--format text|json|markdown|html] [--out <path>] [--strict] [--profile <conformance-profile>]
dpcs inspect <path> [--json] [--format text|json|markdown|html] [--out <path>] [--tui]
dpcs diagnostics <path> [--json] [--format text|json|markdown|html] [--out <path>]
dpcs graph <path> [--json] [--format text|json|markdown|html|mermaid|dot] [--out <path>]
dpcs capabilities <profile> --plan <contract> [--json] [--format text|json|markdown|html] [--out <path>]
dpcs bind <contract> --profile <profile> --target <airflow|dagster|prefect|temporal|kubernetes>
dpcs compatibility <baseline> <candidate> [--json] [--format text|json|markdown|html] [--out <path>]
dpcs tui <path>
dpcs registry validate <path>
dpcs registry serve --root <dir> [--bind host:port] [--token <secret>]
dpcs registry pull --url <base> [--token <secret>] [--cache-dir <dir>]
dpcs registry lookup --url <base> <id> [--version <ver>] [--token <secret>] [--cache-dir <dir>]
dpcs registry publish --url <base> <artifact> [--content <file>] [--token <secret>] [--cache-dir <dir>]
dpcs registry deprecate --url <base> <id> [--version <ver>] [--token <secret>] [--cache-dir <dir>]
dpcs registry retire --url <base> <id> [--version <ver>] [--token <secret>] [--cache-dir <dir>]
dpcs registry cache --dir <dir> [--clear]
dpcs package validate|show|pack|unpack ...
dpcs schema json|openapi ...
dpcs conformance validate <path>
dpcs version [--json]
```

Output modes:

```bash
dpcs validate pipeline.yaml
dpcs validate pipeline.yaml --json
dpcs validate pipeline.yaml --format markdown --out report.md
dpcs validate pipeline.yaml --strict
dpcs inspect pipeline.yaml --format html --out inspect.html
dpcs graph pipeline.yaml --format mermaid --out graph.mmd
dpcs graph pipeline.yaml --format dot
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml
dpcs capabilities orchestrator.capabilities.yaml --plan pipeline.yaml --json
dpcs bind pipeline.yaml --profile orchestrator.capabilities.yaml --target airflow
dpcs bind pipeline.yaml --profile orchestrator.capabilities.yaml --target airflow --out ./out --json
dpcs compatibility baseline.yaml candidate.yaml --json
dpcs inspect pipeline.yaml --tui
dpcs tui pipeline.yaml
dpcs registry validate registry.yaml --json
dpcs conformance validate conformance.profile.yaml --json
dpcs version --json
```

Exit codes:

- `0` valid / capability match ok / bind ok / compatible / registry or conformance ok
- `1` validation, capability, binding, compatibility, registry, or conformance errors (including HTTP 4xx from the registry API)
- `2` parse or IO failure (including registry transport errors, response decode failures, and HTTP 5xx)

Notes:

- `validate`, `diagnostics`, `capabilities`, and `bind` resolve nested DPCS
  Contract References relative to the **input document directory**
  (`ResolveOptions::from_document_path`). Companion ODCS/DTCS files may be absent.
- `validate` and `diagnostics` use the exit codes above.
- `--format` selects the report renderer (`text`, `json`, `markdown`, `html`; `graph` also
  accepts `mermaid` and `dot`). When omitted, output is text. `--json` is a permanent
  alias for `--format json`.
- `--out <path>` writes the report to a file; when omitted, reports go to stdout.
- Severity lines in text mode use ANSI color when stdout is a TTY (honors `NO_COLOR`).
- `validate --profile` applies a validated conformance profile (`requireSecurity`,
  `requireGovernance`, forbidden extension namespaces) via `apply_profile_to_contract`.
- `validate --json` / `--format json` and `diagnostics --json` / `--format json` emit a
  `DiagnosticReport` (processing result, artifact id, implementation metadata, and diagnostics).
  Markdown/HTML use the plain validation diagnostic list.
- `capabilities` parses the contract, runs `plan()`, then `evaluate` against the profile. Exit `0` on match, `1` on plan refusal or capability errors, `2` on parse/I/O.
- `capabilities --json` / `--format json` emits a `CapabilityReport` on both success and match failure (failure includes diagnostics and `missingMandatory`).
- `bind` parses the contract and profile, plans (with document-relative resolve),
  capability-gates, then translates to scaffold artifacts **plus**
  `dpcs_semantics.json`. Writes under `--out` (default `./dpcs-bind-<target>/`).
  Exit `0` on success, `1` on plan/capability/binding errors (including unknown
  `--target`), `2` on parse/I/O or write failure.
- `bind --json` emits a `BindingBundle` on success (and still writes files when `--out` is used or defaulted).
- `--target` accepts `airflow`, `dagster`, `prefect`, `temporal`, `kubernetes`, and alias `k8s`.
- `temporal` and `kubernetes` targets are experimental.
- `compatibility` compares two Pipeline Contracts. Exit `0` when the category is compatible (fully / backward / forward / conditional), `1` when incompatible, `2` on parse/I/O. `--json` / `--format json` emits `CompatibilityReport`.
- `registry validate` validates an in-process registry document (`DPCS-REG-*`).
- `registry serve|pull|lookup|publish|deprecate|retire|cache` use the reference HTTP API (ADR-0005).
  Client commands accept `--cache-dir` for a disk-backed `RegistryCache`. Deprecate/retire
  accept optional `--version` (server uses latest listed row when omitted).
  Publish of an existing `id@version` with **different content** is rejected
  (HTTP 409 / `DPCS-REG-016`); identical content is idempotent. Status changes
  use deprecate/retire, not republish.
- `package` operates on `.dpcspkg` layouts (`DPCS-PKG-*`).
- `schema` emits JSON Schema / OpenAPI under `schemas/`.
- `conformance validate` validates a conformance profile and checks claimed levels against this toolkit.
- `version --json` emits toolkit version, `dpcsSpecVersion`, and the toolkit `ConformanceClaim`.
- Invalid YAML/JSON documents emit Parse-stage diagnostics (`DPCS-PARSE-*`) and exit `2`. With `--json`, the diagnostic report is printed to stdout for `validate`, `diagnostics`, `inspect`, `graph`, `capabilities`, `bind`, `compatibility`, `registry`, and `conformance` failures.
- `inspect` and `graph` always exit `0` after a successful parse and do not fail on validation errors.
- `inspect` reports `valid`, execution-model counts, and `planningRefused` / `stepOrder` when a plan is available.
- `inspect --tui` and `tui` open an interactive inspector (Overview / Steps / Edges / Diagnostics / Plan). Requires a TTY and the `tui` feature (included in `dpcs-cli` / `--features full`). Exit `2` when not a TTY or parse fails.
- `graph` prints topology always; planned `stepOrder` is omitted when planning is refused, and both text/JSON report `planningRefused`. Graph formats `mermaid`/`dot`/`html` export visualization source.
- JSON serialization failures for reports exit with code `2`.
