# DPCS Reference Registry HTTP API

Implementation-defined reference registry protocol for ROADMAP 0.10.

SPEC Chapter 22 defines the registry **document** model and leaves operational
mechanisms implementation-defined. Appendix G may later standardize a protocol.
This document describes the **DPCS reference** HTTP API shipped with toolkit
0.10.0. See [ADR-0005](../adr/ADR-0005-reference-registry-http-api.md).

## Base URL

All paths are relative to a registry base URL, for example `http://127.0.0.1:8080`.

## Endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/v1/health` | Liveness |
| `GET` | `/v1/registry` | Full registry document |
| `GET` | `/v1/artifacts` | Discovery (`?type=` / `?status=`) |
| `GET` | `/v1/artifacts/{id}` | Lookup (`?version=`) |
| `GET` | `/v1/artifacts/{id}/content` | Fetch payload |
| `PUT` | `/v1/artifacts/{id}` | Publish / update |
| `POST` | `/v1/artifacts/{id}/deprecate` | Deprecate (`?version=`) |
| `POST` | `/v1/artifacts/{id}/retire` | Retire (`?version=`) |

Path and query values are percent-encoded by `RegistryClient`. Artifact ids and
versions used as content filenames must be safe path segments
(`[A-Za-z0-9._+-]`, including SemVer `+build` metadata).

## Auth

Mutating operations require `Authorization: Bearer <token>` when the server is
started with `--token`. Content payloads support `contentEncoding=utf-8` only.

## Validation

Document validation uses `validate_registry` (`DPCS-REG-*`). Client/transport
failures use `DPCS-REGC-*`.

## CLI

```bash
dpcs registry serve --root /tmp/reg --bind 127.0.0.1:8080 --token secret
dpcs registry pull --url http://127.0.0.1:8080 --cache-dir /tmp/reg-cache
dpcs registry lookup --url http://127.0.0.1:8080 demo --version 0.1.0 --cache-dir /tmp/reg-cache
dpcs registry publish --url http://127.0.0.1:8080 artifact.yaml --content payload.yaml --token secret
dpcs registry deprecate --url http://127.0.0.1:8080 demo --version 0.1.0 --token secret
dpcs registry retire --url http://127.0.0.1:8080 demo --version 0.1.0 --token secret
```

## OpenAPI

Generated at `schemas/registry.openapi.json` via:

```bash
dpcs schema openapi --kind registry --out schemas
```
