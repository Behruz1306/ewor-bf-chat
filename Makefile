.PHONY: all build gen run-server run-client demo test clean review

all: build gen

build:
	cargo build --release

gen:
	cargo run --release --bin bf-gen

run-server: gen
	./target/release/bf-run --bfa programs/server.bf

run-client: gen
	./target/release/bf-run --bfa programs/client.bf

demo: build gen
	bash scripts/demo.sh

test: build gen
	cargo test

review: build gen test
	@echo "ready"

clean:
	cargo clean
