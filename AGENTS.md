# AI Coding Instructions for cmvm

## Project Overview

`cmvm` is a CMake Version Manager — a Rust CLI tool that downloads, installs, and switches between multiple CMake versions on macOS and Linux.

## Repository Structure

```
src/
  main.rs        – CLI entry point (clap-based argument parsing)
  commands.rs    – Top-level command implementations
  releases.rs    – Fetching and caching GitHub release metadata
  versions.rs    – Version model (parsing, listing, activation)
  package.rs     – Downloading, decompressing, and copying CMake archives
  platform.rs    – Platform detection (macOS / Linux) and asset filtering
  storage.rs     – Filesystem paths via the `Storage` trait
  cache.rs       – Cache bootstrapping and directory helpers
  http.rs        – HTTP helpers (reqwest blocking client)
  constants.rs   – Shared constants (base URL, platform list, etc.)
  types.rs       – Shared type aliases
```

## Build & Test

```bash
# Build
cargo build

# Run all tests
cargo test

# Build release binary
cargo build --release
```

CI runs `cargo build --verbose` and `cargo test --verbose` on both `ubuntu-latest` and `macos-latest`.

## Key Conventions

- **Error handling**: use `anyhow::Result` and `anyhow::bail!` throughout. Avoid `unwrap()` except where already present in existing code.
- **Dependency injection**: the `Storage` trait is injected into every command so that tests can provide a mock implementation.
- **Platform support**: only `macos` and `linux` are supported. Check `platform::is_supported_platform()` before performing OS-specific operations.
- **No `async`**: all I/O (HTTP, filesystem) is synchronous (`reqwest` blocking feature).
- **Rust edition**: 2021.
- **Formatting**: follow standard `rustfmt` conventions (run `cargo fmt` before committing).
- **Linting**: the project should pass `cargo clippy` without warnings.

## Adding a New Command

1. Add a variant to the `CliCommands` enum in `src/main.rs` with a doc-comment that becomes the help text.
2. Implement the corresponding method on `Commands` in `src/commands.rs`.
3. Wire the new variant in the `match` block inside `main()`.
4. Add unit tests in the relevant module (or in `src/commands.rs`).

## Testing Guidelines

- Unit tests live in `#[cfg(test)] mod tests` blocks within the same file as the code under test.
- Use the `pretty_assertions` crate for clearer diff output in assertions.
- Mock the `Storage` trait for tests that touch the filesystem.
- Platform-specific tests should be gated with `#[cfg(target_os = "...")]`.
