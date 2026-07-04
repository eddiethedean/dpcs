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
