# CLI Specification

Binary name:

```bash
dpcs
```

Initial commands:

```bash
dpcs validate <path>
dpcs inspect <path>
dpcs diagnostics <path>
dpcs graph <path>
dpcs version
```

Output modes:

```bash
dpcs validate pipeline.yaml
dpcs validate pipeline.yaml --json
dpcs validate pipeline.yaml --strict
```

Exit codes:

- `0` valid
- `1` validation errors
- `2` parse or IO failure

Notes:

- `validate` and `diagnostics` use the exit codes above.
- `inspect` and `graph` always exit `0` after a successful parse; validity is reported in the output (`valid` field / summary) and does not change the exit code.
- JSON serialization failures for reports exit with code `2`.
