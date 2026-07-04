.PHONY: fmt lint test build check ci

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-features

build:
	cargo build --release

check: fmt lint test

ci: lint test
	cargo build --release
