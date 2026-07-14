import json
from pathlib import Path

import dpcs

ROOT = Path(__file__).resolve().parents[3]


def test_version():
    assert dpcs.version()
    assert dpcs.dpcs_spec_version() == "1.0.0-draft"


def test_validate_minimal():
    source = (ROOT / "examples/minimal.dpcs.yaml").read_text()
    report = dpcs.validate_yaml(source)
    assert "diagnostics" in report
