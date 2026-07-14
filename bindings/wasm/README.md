# dpcs (WebAssembly)

JavaScript/TypeScript bindings for the DPCS Rust toolkit via `wasm-bindgen`.

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

Build: `wasm-pack build --target bundler --out-dir pkg --out-name dpcs`
