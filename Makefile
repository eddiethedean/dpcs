.PHONY: fmt fmt-check lint test build examples check ci schema docs docs-site bench conformance diagnostics-catalog

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

build:
	cargo build -p dpcs-cli --release

docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

bench:
	cargo bench -p dpcs --features parallel --bench performance

conformance:
	cargo test -p dpcs --all-features --test conformance --test conformance_appendix_e

diagnostics-catalog:
	python3 -c "import json,re,pathlib; root=pathlib.Path('crates/dpcs/src'); ids=sorted({m.group(1) for p in root.rglob('*.rs') for m in re.finditer(r'\"(DPCS-[A-Z]+-\\d+)\"', p.read_text())}); fam={}; \
[fam.setdefault(i.rsplit('-',1)[0], []).append(i) for i in ids]; pathlib.Path('docs/diagnostics.catalog.json').write_text(json.dumps({'generatedFrom':'crates/dpcs/src','count':len(ids),'ids':ids,'families':fam}, indent=2)+'\n')"

docs-site:
	python3 -m pip install -q -r docs/requirements.txt
	mkdocs build --strict

schema:
	cargo run -p dpcs-cli --release -- schema json --out schemas
	cargo run -p dpcs-cli --release -- schema openapi --kind all --out schemas

examples:
	cargo run -p dpcs-cli --release -- validate examples/minimal.dpcs.yaml
	cargo run -p dpcs-cli --release -- validate examples/with_execution.dpcs.yaml
	cargo run -p dpcs-cli --release -- validate examples/with_security_governance.dpcs.yaml
	cargo run -p dpcs-cli --release -- capabilities examples/orchestrator.capabilities.yaml --plan examples/with_execution.dpcs.yaml
	cargo run -p dpcs-cli --release -- bind examples/with_execution.dpcs.yaml --profile examples/orchestrator.capabilities.yaml --target airflow --out /tmp/dpcs-bind-smoke
	cargo run -p dpcs-cli --release -- compatibility examples/compatibility/baseline.dpcs.yaml examples/compatibility/candidate_compatible.dpcs.yaml
	cargo run -p dpcs-cli --release -- registry validate examples/registry.yaml
	cargo run -p dpcs-cli --release -- conformance validate examples/conformance.profile.yaml
	cargo run -p dpcs-cli --release -- package validate examples/packages/minimal.dpcspkg
	cargo run -p dpcs-cli --release -- inspect examples/minimal.dpcs.yaml --format markdown --out /tmp/dpcs-inspect.md
	cargo run -p dpcs-cli --release -- graph examples/with_execution.dpcs.yaml --format mermaid --out /tmp/dpcs-graph.mmd
	cargo run -p dpcs-cli --release -- graph examples/with_execution.dpcs.yaml --format html --out /tmp/dpcs-graph.html
	test -s /tmp/dpcs-inspect.md
	test -s /tmp/dpcs-graph.mmd
	grep -q flowchart /tmp/dpcs-graph.mmd
	grep -q Pipeline /tmp/dpcs-inspect.md

check: fmt-check lint test conformance

ci: lint test conformance examples build
