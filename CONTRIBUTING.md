# Contributing

Thanks for helping build the DPCS reference implementation.

## Ground rules

1. Treat [`SPEC.md`](SPEC.md) as the authoritative source of truth.
2. If a design doc conflicts with `SPEC.md`, follow `SPEC.md`.
3. Prefer the smallest conservative behavior when the specification is ambiguous.
4. Add a `TODO` referencing the relevant SPEC section when deferring behavior.
5. Do not implement execution runtimes; binding scaffolds and Ch 18–25 document
   models are shipped. Registry networking uses the reference HTTP API (ADR-0005).

## Releases

Tagged releases use `.github/workflows/release.yml`:

1. Push a version tag matching `v*.*.*` (for example `v0.11.0`).
2. The release workflow runs the same CI checks as pull requests (`ci-checks.yml`).
3. After checks pass, it publishes:
   - Rust crates `dpcs` and `dpcs-cli` to crates.io (`CARGO_REGISTRY_TOKEN`)
   - Python package `dpcs` to PyPI (`PYPI_API_TOKEN`)
   - WASM/JS package `@eddiethedean/dpcs` to npm (`NPM_TOKEN`)
   - WASM package `eddiethedean/dpcs` to Wasmer (`WASM_TOKEN`)
4. Create a GitHub Release for the tag using the matching section from [`CHANGELOG.md`](CHANGELOG.md).

To re-publish only npm / Wasmer after a tag (for example after a build-tool fix),
use `.github/workflows/publish-wasm.yml` (`workflow_dispatch`). It builds from
the checked-out `main` ref and requires the workspace version to match the
selected tag (for example `v0.11.0`). See [`docs/BINDINGS.md`](docs/BINDINGS.md).

## Workflow

1. Create a focused branch.
2. Implement the change with tests.
3. Run:

   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   # or: make ci
   ```

4. Update docs when public behavior changes.
5. Open a pull request against `main`.

## Project map

| Path | Purpose |
| --- | --- |
| `SPEC.md` | Normative specification |
| `ROADMAP.md` | Release plan |
| `crates/dpcs` | Core library |
| `crates/dpcs-cli` | `dpcs` CLI binary |
| `bindings/python` | PyO3 / maturin → PyPI `dpcs` (outside workspace) |
| `bindings/wasm` | wasm-bindgen → npm `@eddiethedean/dpcs`, Wasmer `eddiethedean/dpcs` |
| `schemas/` | Generated JSON Schema and OpenAPI artifacts |
| `examples/` | Example contracts, packages, and profiles |
| `docs/` | Design guides (including [`docs/BINDINGS.md`](docs/BINDINGS.md)) |
| `adr/` | Architecture decisions |

## Development setup

```bash
rustup toolchain install stable
cargo test --workspace --all-features
```

MSRV is **1.88**. Local parity with CI: `make ci`. Rust API docs: `make docs`.
Guide site (MkDocs / Read the Docs): `make docs-site` or `mkdocs serve` after
`pip install -r docs/requirements.txt`. Published guides:
[dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/).

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
