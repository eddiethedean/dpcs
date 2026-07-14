# dpcs (Python)

Python bindings for the DPCS Rust reference toolkit (ROADMAP 0.10).

Published on PyPI as [`dpcs`](https://pypi.org/project/dpcs/). See
[`docs/BINDINGS.md`](../../docs/BINDINGS.md).

```bash
pip install dpcs
# or from this repo (requires a venv for `maturin develop`):
cd bindings/python
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest -q
```

```python
import dpcs

report = dpcs.validate_yaml(open("pipeline.dpcs.yaml").read())
assert report["diagnostics"] == [] or all(
    d["severity"] != "error" for d in report["diagnostics"]
)
print(dpcs.version(), dpcs.dpcs_spec_version())
```
