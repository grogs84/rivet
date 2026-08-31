# Rivet

Rivet is an experimental graph database implementation in Rust.

## Status

Rivet is at the very beginning of development. The first milestone is a minimal Rust application that verifies the project builds and runs correctly.

## Run

```bash
cargo run
```

Expected output:

```text
Hello from Rivet!
```

## Development

Format, lint, and test the project with:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
