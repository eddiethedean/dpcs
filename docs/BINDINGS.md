# Language Bindings and Distribution

Ecosystem packages for ROADMAP 0.10.0. The Rust `dpcs` library remains the
canonical implementation; bindings wrap a subset of parse / validate / plan /
capabilities / bind / compatibility / registry / conformance entry points.

| Channel | Package | Install |
| --- | --- | --- |
| crates.io | [`dpcs`](https://crates.io/crates/dpcs), [`dpcs-cli`](https://crates.io/crates/dpcs-cli) | `cargo install dpcs-cli` |
| PyPI | [`dpcs`](https://pypi.org/project/dpcs/) | `pip install dpcs` |
| npm | [`@eddiethedean/dpcs`](https://www.npmjs.com/package/@eddiethedean/dpcs) | `npm install @eddiethedean/dpcs` |
| Wasmer | [`eddiethedean/dpcs`](https://wasmer.io/eddiethedean/dpcs) | see Wasmer registry |

The bare npm name `dpcs` is reserved/blocked by npm similarity rules; the scoped
`@eddiethedean/dpcs` package is what release CI publishes.

## Python (`bindings/python`)

Maturin / PyO3 (`abi3-py39`). Outside the Cargo workspace (`workspace.exclude`).

```bash
pip install dpcs
# local:
cd bindings/python && python -m venv .venv && source .venv/bin/activate
pip install maturin pytest && maturin develop && pytest -q
```

Import the `dpcs` module (same name as the Rust crate; Rust sources use `::dpcs`
to avoid the PyO3 module-name clash).

## WebAssembly (`bindings/wasm`)

`wasm-bindgen` / `wasm-pack`. Workspace member; release builds enable
`wasm-opt` with `--enable-bulk-memory` and `--enable-nontrapping-float-to-int`
(required for Rust 1.87+ WASM feature defaults).

```bash
npm install @eddiethedean/dpcs
# local:
cd bindings/wasm
wasm-pack build --target bundler --release --out-dir pkg --out-name dpcs
npm pkg set name=@eddiethedean/dpcs   # when publishing from pkg/
```

Wasmer publishes the same crate as `eddiethedean/dpcs` with module ABI `none`
(host JS still required for `wasm-bindgen` glue).

## Release automation

| Workflow | When | What |
| --- | --- | --- |
| `release.yml` | Tag `v*.*.*` | crates.io, PyPI, npm (`@eddiethedean/dpcs`), Wasmer (`eddiethedean/dpcs`) after CI |
| `publish-wasm.yml` | Manual `workflow_dispatch` | Re-publish npm and/or Wasmer from `main` (version must match the selected tag) |

Secrets: `CARGO_REGISTRY_TOKEN`, `PYPI_API_TOKEN`, `NPM_TOKEN`, `WASM_TOKEN`.
