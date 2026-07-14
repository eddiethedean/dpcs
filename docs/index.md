# DPCS documentation

[![Guides](https://img.shields.io/badge/guides-readthedocs.io-blue?logo=readthedocs&logoColor=white)](https://dpcs.readthedocs.io/en/latest/)
[![Rust API](https://img.shields.io/docsrs/dpcs?label=rust%20api)](https://docs.rs/dpcs)
[![crates.io](https://img.shields.io/crates/v/dpcs.svg)](https://crates.io/crates/dpcs)

Reference toolkit for the **Data Pipeline Contract Standard (DPCS)**.

Rust API docs live on [docs.rs/dpcs](https://docs.rs/dpcs). The normative
specification is [`SPEC.md`](https://github.com/eddiethedean/dpcs/blob/main/SPEC.md)
in the repository root. This site covers design guides, CLI behavior, bindings,
and contributor-oriented documentation.

## Install

```bash
cargo install dpcs-cli
# or
pip install dpcs
npm install @eddiethedean/dpcs
```

## Quick commands

```bash
dpcs validate pipeline.dpcs.yaml
dpcs inspect pipeline.dpcs.yaml --format markdown
dpcs graph pipeline.dpcs.yaml --format mermaid
dpcs tui pipeline.dpcs.yaml
dpcs version --json
```

## Where to go next

| Topic | Page |
| --- | --- |
| Processing architecture | [Architecture](ARCHITECTURE.md) |
| Library surface | [Public API](PUBLIC_API.md) |
| CLI flags and exit codes | [CLI specification](CLI_SPEC.md) |
| Python / WASM packages | [Bindings](BINDINGS.md) |
| `.dpcspkg` layout | [Package format](PACKAGE_FORMAT.md) |
| Reference registry HTTP API | [Registry API](REGISTRY_API.md) |

Roadmap and changelog:

- [ROADMAP.md](https://github.com/eddiethedean/dpcs/blob/main/ROADMAP.md)
- [CHANGELOG.md](https://github.com/eddiethedean/dpcs/blob/main/CHANGELOG.md)

## Local docs builds

```bash
python -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r docs/requirements.txt
mkdocs serve
```

Rustdoc (API reference only): `make docs`.
