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
| `POST` | `/v1/artifacts/{id}/deprecate` | Deprecate |
| `POST` | `/v1/artifacts/{id}/retire` | Retire |

## Auth

Mutating operations MAY require `Authorization: Bearer <token>` when the server
is started with `--token`.

## Validation

Document validation uses `validate_registry` (`DPCS-REG-*`). Client/transport
failures use `DPCS-REGC-*`.

## CLI

```bash
dpcs registry serve --root /tmp/reg --bind 127.0.0.1:8080 --token secret
dpcs registry pull --url http://127.0.0.1:8080
dpcs registry lookup --url http://127.0.0.1:8080 demo --version 0.1.0
dpcs registry publish --url http://127.0.0.1:8080 artifact.yaml --content payload.yaml --token secret
```

## OpenAPI

Generated at `schemas/registry.openapi.json` via:

```bash
dpcs schema openapi --kind registry --out schemas
```
