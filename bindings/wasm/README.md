# dpcs (WebAssembly)

JavaScript/TypeScript bindings for the DPCS Rust toolkit via `wasm-bindgen`.

Published as:

- npm: [`@eddiethedean/dpcs`](https://www.npmjs.com/package/@eddiethedean/dpcs)
- Wasmer: [`eddiethedean/dpcs`](https://wasmer.io/eddiethedean/dpcs)

See [`docs/BINDINGS.md`](../../docs/BINDINGS.md).

```bash
npm install @eddiethedean/dpcs
# from this repo after wasm-pack build:
# npm install ./bindings/wasm/pkg
```

```js
const dpcs = require("@eddiethedean/dpcs");
const report = dpcs.validate_yaml(yamlSource);
console.log(report.diagnostics);
```

Build:

```bash
wasm-pack build --target bundler --release --out-dir pkg --out-name dpcs
```

Release profiles pass bulk-memory flags to `wasm-opt` (see `Cargo.toml`
`package.metadata.wasm-pack.profile.release`).
