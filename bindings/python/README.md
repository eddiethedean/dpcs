# dpcs (Python)

[![PyPI](https://img.shields.io/pypi/v/dpcs.svg)](https://pypi.org/project/dpcs/)
[![Python](https://img.shields.io/pypi/pyversions/dpcs.svg)](https://pypi.org/project/dpcs/)
[![Guides](https://img.shields.io/badge/guides-readthedocs.io-blue?logo=readthedocs&logoColor=white)](https://dpcs.readthedocs.io/en/latest/)
[![License](https://img.shields.io/pypi/l/dpcs.svg)](https://github.com/eddiethedean/dpcs#license)

Python bindings for the [Data Pipeline Contract Standard (DPCS)](https://github.com/eddiethedean/dpcs)
Rust reference toolkit. Parse, validate, plan, compare, and bind pipeline
contracts from Python via PyO3.

| | |
| --- | --- |
| Package | [`dpcs`](https://pypi.org/project/dpcs/) on PyPI |
| Toolkit | `0.11.x` (ROADMAP developer-experience release) |
| Spec | `1.0.0-draft` |
| Requires | Python ≥ 3.9 (abi3) |
| Guides | [dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/) |
| Bindings notes | [docs/BINDINGS.md](https://github.com/eddiethedean/dpcs/blob/main/docs/BINDINGS.md) |

## Install

```bash
pip install dpcs
```

## Quick start

```python
from pathlib import Path

import dpcs

source = Path("pipeline.dpcs.yaml").read_text()
report = dpcs.validate_yaml(source)

errors = [d for d in report["diagnostics"] if d.get("severity") == "error"]
assert not errors, errors

print(dpcs.version(), dpcs.dpcs_spec_version())
# or from a path (YAML or JSON):
# report = dpcs.validate_file("pipeline.dpcs.yaml")
```

## API

Functions return JSON-compatible `dict` / `list` values (serialized from the
Rust types) unless noted.

| Function | Purpose |
| --- | --- |
| `version()` / `__version__` | Toolkit version string |
| `dpcs_spec_version()` | Spec version (`1.0.0-draft`) |
| `parse_yaml_str(source)` / `parse_json_str(source)` | Parse a contract document |
| `validate_yaml(source)` / `validate_file(path)` | Validate YAML/JSON (file auto-detects) |
| `plan_yaml(source)` | Build a pipeline plan (or validation report on refusal) |
| `evaluate_capabilities(profile_yaml, contract_yaml)` | Check orchestrator capabilities |
| `bind_yaml(contract_yaml, profile_yaml, target)` | Emit scaffolding (`airflow`, `dagster`, `prefect`, `temporal`, `kubernetes`, …) |
| `compare_contract_yaml(baseline, candidate)` | Compatibility comparison |
| `validate_registry_yaml(source)` | Validate an artifact registry document |
| `validate_conformance_profile_yaml(source)` | Validate a conformance profile |
| `to_yaml_str(contract_json)` / `to_json_str(contract_yaml)` | Format conversion |

CLI, report formats, and TUI are Rust-first (`dpcs-cli`); this package wraps the
core library entry points. Full toolkit docs:
[dpcs.readthedocs.io](https://dpcs.readthedocs.io/en/latest/).

## Develop from source

Requires a virtualenv for `maturin develop`. The Python crate sits outside the
Cargo workspace (`workspace.exclude`).

```bash
cd bindings/python
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest -q
```

## License

Apache-2.0 OR MIT — same as the main [dpcs](https://github.com/eddiethedean/dpcs) repository.
