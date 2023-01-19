# Standalone development workflow targets
# Running those inside existing workspace will break due to Cargo unable to support nested worksapce

Cargo.toml: Cargo.dev.toml
	cp Cargo.dev.toml Cargo.toml

dev-format: Cargo.toml
	cargo fmt --all

dev-format-check: Cargo.toml
	cargo fmt --all -- --check

# needs to use run.sh to check individual projects because
#   --no-default-features is not allowed in the root of a virtual workspace
dev-check: Cargo.toml check

dev-check-tests: Cargo.toml
	cargo check --tests --all

dev-test: Cargo.toml
	cargo test --all
