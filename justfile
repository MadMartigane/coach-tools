# Run from the repository root. Run `just` alone to list the recipes.

default:
    @just --list

# Serve the app locally with live reload.
dev:
    trunk serve --config app

# Serve the app on the local network, for testing on a phone.
dev-lan:
    trunk serve --config app --address 0.0.0.0

# Build the production bundle into app/dist/.
build:
    trunk build --release --config app

# Deploy to production. Runs on the production machine only; requires branch main.
prod:
    scripts/deploy.sh prod

# Deploy to pre-production. Runs on the production machine only; requires branch develop.
pre-prod:
    scripts/deploy.sh pre-prod

# Format all Rust code in place.
fmt:
    cargo fmt --all

# Run the unit tests of all crates.
test:
    cargo test --workspace

# Format check, lints for the host and the browser target, then tests.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
    cargo test --workspace
