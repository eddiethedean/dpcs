# DPCS Pipeline Package Format

Implementation-defined pipeline package layout for ROADMAP 0.10 (Appendix G).

## Layout

```text
name.dpcspkg/
  manifest.yaml
  artifacts/
    contracts/...
    plans/...
    profiles/...
    registry.yaml   # optional
```

Directory packages may also be zipped (`.dpcspkg.zip`).

## Manifest

`PackageManifest` fields:

| Field | Required | Description |
| --- | --- | --- |
| `id` | yes | Stable package identifier |
| `version` | yes | SemVer-compatible package version |
| `dpcsVersion` | yes | Supported DPCS specification version |
| `name` | no | Human-readable name |
| `description` | no | Description |
| `artifacts[]` | no | Indexed artifacts with `id`, `type`, `version`, `path` |

Artifact paths are relative to the package root and must not contain `..`.

## CLI

```bash
dpcs package validate examples/packages/minimal.dpcspkg
dpcs package show examples/packages/minimal.dpcspkg --json
dpcs package pack examples/packages/minimal.dpcspkg --archive /tmp/minimal.dpcspkg.zip
dpcs package unpack /tmp/minimal.dpcspkg.zip /tmp/out
```

## Library

```rust
use dpcs::{pack, unpack, validate_package, resolve_artifact};
```
