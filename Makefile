build:
	cargo build

test: build
	cargo test -- --nocapture

lint: format clippy check

vuln: audit outdated

format:
	cargo fmt --all -- --check

fix:
	cargo fix

clippy:
	cargo clippy -- -D warnings

check:
	cargo check -v

audit:
	cargo install --locked cargo-audit
	cargo audit

outdated:
	cargo install --locked cargo-outdated
	cargo outdated -v

.PHONY: build test format fix clippy check audit outdated lint vuln
