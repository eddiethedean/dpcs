.PHONY: fmt fmt-check lint test build examples check ci schema docs

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

check: fmt-check lint test

ci: lint test examples build
