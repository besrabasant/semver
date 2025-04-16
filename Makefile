# Project metadata
BINARY_NAME := semver
BUILD_DIR := ./target
TMP_DIR := ./tmp-test

# Default target
all: build

## Build the semver binary
build:
	cargo build --release

## Run all tests
test:
	cargo test && rm -rf $(TMP_DIR)

## Clean target and tmp directories
clean:
	cargo clean
	rm -rf $(TMP_DIR)

## Format all Rust code
fmt:
	cargo fmt

## Run the built binary
run:
	cargo run

## Build and run the release binary
release-run:
	$(BUILD_DIR)/release/$(BINARY_NAME)

.PHONY: all build test clean fmt run release-run
