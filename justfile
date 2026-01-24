# https://just.systems

# Show available recipes
default:
    @just --unsorted --list

# Run all checks
[group('dev')]
check: fmt-check lint test build

# Build WASM extension
[group('dev')]
build:
    cargo build --target wasm32-wasip2 --release

# Run tests
[group('dev')]
test:
    cargo test

# Format with nightly rustfmt (for unstable options)
[group('format')]
fmt:
    cargo +nightly fmt
    dprint fmt

# Check formatting
[group('format')]
fmt-check:
    cargo +nightly fmt --check
    dprint check

# Run clippy
[group('lint')]
lint:
    cargo clippy --target wasm32-wasip2 -- -D warnings
