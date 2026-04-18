SHELL := /usr/bin/env bash

.PHONY: check build test fmt clippy clean deploy-test

check:
	cargo check --all

build:
	stellar contract build

test:
	cargo test --all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all -- -D warnings

clean:
	cargo clean

deploy-test:
	bash scripts/deploy_testnet.sh
