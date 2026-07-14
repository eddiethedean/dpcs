.PHONY: fmt fmt-check lint test build examples check ci

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-features

build:
	cargo build --release

examples:
	cargo run --release -- validate examples/minimal.dpcs.yaml
	cargo run --release -- validate examples/with_execution.dpcs.yaml
	cargo run --release -- capabilities examples/orchestrator.capabilities.yaml --plan examples/with_execution.dpcs.yaml
	cargo run --release -- bind examples/with_execution.dpcs.yaml --profile examples/orchestrator.capabilities.yaml --target airflow --out /tmp/dpcs-bind-smoke

check: fmt-check lint test

ci: lint test examples build
