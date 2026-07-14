# Proposed Crate Layout

```text
.
├── Cargo.toml                 # workspace (excludes bindings/python)
├── SPEC.md
├── README.md
├── ROADMAP.md
├── crates/
│   ├── dpcs/                  # core library
│   └── dpcs-cli/              # dpcs binary
├── bindings/
│   ├── python/                # maturin / PyO3 → PyPI `dpcs`
│   └── wasm/                  # wasm-bindgen → npm + Wasmer
├── schemas/                   # generated JSON Schema + OpenAPI
├── examples/
├── docs/
│   ├── BINDINGS.md            # language packages and publish channels
│   ├── PACKAGE_FORMAT.md
│   └── REGISTRY_API.md
└── adr/
```

Keep the modules aligned with `SPEC.md`. See [`BINDINGS.md`](BINDINGS.md) for
install and release channel names.
