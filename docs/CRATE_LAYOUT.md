# Proposed Crate Layout

```text
.
├── Cargo.toml                 # workspace
├── SPEC.md
├── README.md
├── ROADMAP.md
├── crates/
│   ├── dpcs/                  # core library
│   └── dpcs-cli/              # dpcs binary
├── bindings/
│   ├── python/                # maturin / PyO3
│   └── wasm/                  # wasm-bindgen
├── schemas/                   # generated JSON Schema + OpenAPI
├── examples/
├── docs/
└── adr/
```

Keep the modules aligned with `SPEC.md`.
