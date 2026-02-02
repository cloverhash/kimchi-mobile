# Contributing to Kimchi Mobile

Thank you for your interest in contributing to Kimchi Mobile!

## Getting Started

1. Fork the repository
2. Clone your fork
3. Run the setup script:
   ```bash
   ./scripts/setup.sh
   ```

## Development Workflow

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run a specific test
cargo test test_name
```

### Code Style

We use `rustfmt` for formatting:

```bash
# Check formatting
cargo fmt --all -- --check

# Fix formatting
cargo fmt --all
```

Run `clippy` for lints:

```bash
cargo clippy --all-targets --all-features
```

### Building for Platforms

```bash
# Android
./scripts/build-android.sh

# iOS (macOS only)
./scripts/build-ios.sh

# WebAssembly
./scripts/build-wasm.sh
```

## Project Structure

- `kimchi-prover/` - Core Rust prover library
- `kimchi-ffi/` - FFI bindings for Android/iOS (UniFFI)
- `kimchi-wasm/` - WebAssembly bindings
- `kotlin/` - Android Kotlin wrapper
- `swift/` - iOS Swift wrapper
- `examples/` - Example circuits and usage

## Adding a New Circuit

1. Create a new file in `kimchi-prover/src/circuits/`
2. Implement the circuit logic with gates and witness generation
3. Export via FFI in `kimchi-ffi/src/lib.rs`
4. Add platform wrappers in `kotlin/` and `swift/`
5. Add tests

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Ensure tests pass: `cargo test --workspace`
4. Ensure formatting is correct: `cargo fmt --all`
5. Submit a pull request

## Reporting Issues

Please include:
- Description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Platform/device information
- Rust version (`rustc --version`)
