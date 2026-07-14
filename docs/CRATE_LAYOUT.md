# Crate layout

```text
.
├── Cargo.toml                 # workspace (excludes bindings/python)
├── SPEC.md
├── README.md
├── ROADMAP.md
├── crates/
│   ├── dpcs/                  # core library
│   │   └── src/
│   │       ├── model/         # COM
│   │       ├── parser/
│   │       ├── validation/
│   │       ├── resolve/       # Ch 7 reference resolution + nesting (0.13)
│   │       ├── plan/
│   │       ├── capabilities/
│   │       ├── binding/       # scaffolds + dpcs_semantics.json
│   │       ├── compatibility/
│   │       ├── conformance/
│   │       ├── registry_net/  # reference HTTP (optional features)
│   │       ├── package/
│   │       ├── report/
│   │       ├── schema/
│   │       └── cli/
│   └── dpcs-cli/              # thin dpcs binary
├── bindings/
│   ├── python/                # maturin / PyO3 → PyPI `dpcs`
│   └── wasm/                  # wasm-bindgen → npm + Wasmer
├── schemas/                   # generated JSON Schema + OpenAPI
├── examples/
├── docs/                      # MkDocs guides (incl. SPEC_COVERAGE.md)
└── adr/
```

Keep modules aligned with `SPEC.md`. Install/publish channels:
[`BINDINGS.md`](BINDINGS.md). Completeness checklist:
[`SPEC_COVERAGE.md`](SPEC_COVERAGE.md).
