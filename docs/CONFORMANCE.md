# Conformance

SPEC Chapter 23 defines conformance levels. This toolkit claims
`CompleteImplementation` via `toolkit_claim()` / `implemented_levels()` when the
Appendix E suite is green.

## Run the suite

```bash
make conformance
# or
cargo test -p dpcs --all-features --test conformance --test conformance_appendix_e
```

## Profile example

```bash
dpcs conformance validate examples/conformance.profile.yaml
```

## Coverage map

Chapter-level status lives in [`SPEC_COVERAGE.md`](SPEC_COVERAGE.md).
Diagnostic identifiers are listed in [`diagnostics.catalog.json`](diagnostics.catalog.json).
