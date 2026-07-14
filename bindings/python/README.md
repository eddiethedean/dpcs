# dpcs (Python)

Python bindings for the DPCS Rust reference toolkit (ROADMAP 0.10).

```bash
pip install dpcs
# or from this repo:
pip install ./bindings/python
```

```python
import dpcs

report = dpcs.validate_yaml(open("pipeline.dpcs.yaml").read())
assert report["diagnostics"] == [] or all(d["severity"] != "error" for d in report["diagnostics"])
```
