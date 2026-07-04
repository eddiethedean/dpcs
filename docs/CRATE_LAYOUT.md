# Proposed Crate Layout

Recommended initial layout:

```text
.
├── Cargo.toml
├── SPEC.md
├── README.md
├── ROADMAP.md
├── LICENSE-APACHE
├── LICENSE-MIT
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── bin/dpcs.rs
│   ├── model/
│   ├── parser/
│   ├── validation/
│   ├── diagnostics/
│   ├── plan/
│   ├── capabilities/
│   ├── binding/
│   └── cli/
├── examples/
├── tests/fixtures/
├── docs/
└── adr/
```

Keep the modules aligned with `SPEC.md`.
