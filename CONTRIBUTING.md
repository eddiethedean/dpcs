# Contributing

Thanks for helping build the DPCS reference implementation.

## Ground rules

1. Treat [`SPEC.md`](SPEC.md) as the authoritative source of truth.
2. If a design doc conflicts with `SPEC.md`, follow `SPEC.md`.
3. Prefer the smallest conservative behavior when the specification is ambiguous.
4. Add a `TODO` referencing the relevant SPEC section when deferring behavior.
5. Do not implement orchestrator binding or execution runtimes until roadmap 0.8.0.

## Development setup

```bash
rustup toolchain install stable
cargo test --all-features
```

MSRV is **1.74**.

## Workflow

1. Create a focused branch.
2. Implement the change with tests.
3. Run:

   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   ```

4. Update docs when public behavior changes.
5. Open a pull request against `main`.

## Project map

| Path | Purpose |
| --- | --- |
| `SPEC.md` | Normative specification |
| `ROADMAP.md` | Release plan |
| `src/model` | Canonical Object Model |
| `src/parser` | YAML/JSON parsing |
| `src/validation` | Phase-based validation |
| `src/diagnostics` | Diagnostic types and reports |
| `tests/fixtures` | Conformance-oriented fixtures |
| `docs/` | Design guides |
| `adr/` | Architecture decisions |

## Validation diagnostics

New validation rules should:

- use stable ids (`DPCS-<AREA>-<NNN>`)
- set severity, stage, and category
- include an `object_ref` when possible
- include remediation guidance when actionable
- remain deterministic across runs

## Commit style

Use concise, present-tense commit messages that explain why the change exists:

```text
Add graph cycle detection to validation phase
```

## Code of conduct expectations

Be respectful, assume good intent, and keep review feedback specific and actionable.
