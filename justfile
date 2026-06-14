set shell := ["bash", "-uc"]
set dotenv-load

# List available recipes
default:
    @just --list

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the web service
run:
    cargo run

# Fast compilation check (no binary produced)
check:
    cargo check --all-targets

# Run tests
test:
    cargo nextest run --no-fail-fast

# Apply autofixes from linters
lint-fix:
    cargo fmt --all
    cargo clippy --allow-dirty --all-targets --all-features --fix

# Run Clippy (warnings are errors)
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# Generate and open docs
doc:
    cargo doc --all --open

full-doc:
    cargo doc --all --open --all-features

# Remove build artifacts
clean:
    cargo clean

# Full CI pipeline: lint, then test
ci: lint test
