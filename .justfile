_help:
	just -l

# Run all tests using nextest.
test:
	cargo nextest run

coverage:
	cargo llvm-cov --all-targets --workspace --summary-only

# Run the nightly formatter
fmt:
	cargo +nightly fmt

# Run the same checks we run in CI. Requires nightly.
ci: test fmt
	cargo clippy --all-targets
	cargo test --doc

# Install required tools
setup:
	brew tap ceejbot/tap
	brew install cargo-nextest tomato semver-bump
	rustup install nightly

# Tag a new version for release.
version BUMP:
	#!/usr/bin/env bash
	set -e
	current=$(tomato get package.version Cargo.toml)
	version=$(semver-bump {{BUMP}} "$current")
	tomato set package.version "$version" Cargo.toml &> /dev/null
	cargo generate-lockfile
	git commit Cargo.toml -m "v${version}"
	git tag "v${version}"
	echo "Release tagged for version v${version}"

# publish to crates.io
release:
	cargo publish
