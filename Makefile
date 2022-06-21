.PHONY: build
build:
	cargo build

.PHONY: format
format:
	cargo fmt

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test
