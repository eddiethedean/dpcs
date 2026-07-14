# dpcs (WebAssembly)

[![npm](https://img.shields.io/npm/v/@eddiethedean/dpcs.svg)](https://www.npmjs.com/package/@eddiethedean/dpcs)
[![Wasmer](https://img.shields.io/badge/wasmer-eddiethedean%2Fdpcs-blue)](https://wasmer.io/eddiethedean/dpcs)
[![Guides](https://img.shields.io/badge/guides-readthedocs.io-blue?logo=readthedocs&logoColor=white)](https://dpcs.readthedocs.io/en/latest/)
[![License](https://img.shields.io/npm/l/@eddiethedean/dpcs.svg)](https://github.com/eddiethedean/dpcs#license)

JavaScript/TypeScript bindings for the [Data Pipeline Contract Standard (DPCS)](https://github.com/eddiethedean/dpcs)
Rust reference toolkit via `wasm-bindgen`. Parse, validate, plan, compare, and
bind pipeline contracts in the browser or Node.

| | |
| --- | --- |
| npm | [`@eddiethedean/dpcs`](https://www.npmjs.com/package/@eddiethedean/dpcs) |
| Wasmer | [`eddiethedean/dpcs`](https://wasmer.io/eddiethedean/dpcs) |
| Toolkit | `0.13.x` (ROADMAP reference-implementation release) |
| Spec | `1.0.0-draft` |
| Guides | [dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/) |
| Bindings notes | [docs/BINDINGS.md](https://github.com/eddiethedean/dpcs/blob/main/docs/BINDINGS.md) |

The bare npm name `dpcs` is reserved under npm similarity rules; release CI
publishes the scoped `@eddiethedean/dpcs` package. Wasmer module ABI is `none`
(host JS still required for `wasm-bindgen` glue).

## Install

```bash
npm install @eddiethedean/dpcs
# after a local wasm-pack build:
# npm install ./bindings/wasm/pkg
```

## Quick start

```js
import init, * as dpcs from "@eddiethedean/dpcs";

await init(); // required for some targets / bundlers

const report = dpcs.validate_yaml(yamlSource);
const errors = (report.diagnostics || []).filter((d) => d.severity === "error");
if (errors.length) {
  throw new Error(JSON.stringify(errors));
}

console.log(dpcs.version(), dpcs.dpcs_spec_version());
```

CommonJS (Node / smoke style):

```js
const dpcs = require("@eddiethedean/dpcs");
const report = dpcs.validate_yaml(yamlSource);
console.log(report.diagnostics);
```

## API

Exported functions return plain JS objects (serde → `JsValue`) unless noted.
Parse failures reject / throw as strings.

| Function | Purpose |
| --- | --- |
| `version()` | Toolkit version string |
| `dpcs_spec_version()` | Spec version (`1.0.0-draft`) |
| `validate_yaml(source)` / `validate_json(source)` | Validate a contract document |
| `plan_yaml(source)` | Build a pipeline plan (or validation report on refusal) |
| `evaluate_capabilities(profile_yaml, contract_yaml)` | Check orchestrator capabilities |
| `bind_yaml(contract_yaml, profile_yaml, target)` | Emit scaffolding (`airflow`, `dagster`, `prefect`, `temporal`, `kubernetes`, …) |
| `compare_contract_yaml(baseline, candidate)` | Compatibility comparison |
| `validate_registry_yaml(source)` | Validate an artifact registry document |
| `contract_to_json(source_yaml)` | Convert YAML contract → JSON string |

CLI, report formats, and TUI are Rust-first (`dpcs-cli`); this package wraps the
core library entry points. Full toolkit docs:
[dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/).

## Develop from source

```bash
cd bindings/wasm
wasm-pack build --target bundler --release --out-dir pkg --out-name dpcs
# CI also exercises: --target nodejs --dev
# Wasmer publish uses: --target web --release
```

Release profiles pass `--enable-bulk-memory` and
`--enable-nontrapping-float-to-int` to `wasm-opt` (required for Rust 1.87+ WASM
defaults). See `Cargo.toml` `package.metadata.wasm-pack.profile.release`.

After building for npm, set the scoped name before publish:

```bash
cd pkg && npm pkg set name=@eddiethedean/dpcs
```

## License

Apache-2.0 OR MIT — same as the main [dpcs](https://github.com/eddiethedean/dpcs) repository.
