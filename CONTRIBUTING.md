# Contributing to cmvm

Thank you for your interest in contributing to cmvm! This guide covers everything you need to get started.

## Table of contents

- [Getting started](#getting-started)
- [Building the project](#building-the-project)
- [Running tests](#running-tests)
- [Code style](#code-style)
- [Submitting changes](#submitting-changes)
- [Reporting issues](#reporting-issues)
- [Review and merge process](#review-and-merge-process)
- [Resources for Rust beginners](#resources-for-rust-beginners)

---

## Getting started

1. **Fork** the repository on GitHub and clone your fork:

   ```bash
   git clone https://github.com/<your-username>/cmvm.git
   cd cmvm
   ```

2. Make sure you have the Rust toolchain installed. The recommended way is via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Confirm your setup:

   ```bash
   cargo --version   # should print cargo 1.x.x or later
   rustc --version   # should print rustc 1.x.x or later
   ```

## Building the project

```bash
# Debug build (fast compile, includes debug symbols)
cargo build

# Release build (optimized binary)
cargo build --release
```

The compiled binary is placed in `target/debug/cmvm` or `target/release/cmvm`.

## Running tests

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test <test_name>

# Run tests with output shown even when they pass
cargo test -- --nocapture
```

Tests are co-located with the source file they cover inside `#[cfg(test)] mod tests` blocks. Platform-specific tests are gated with `#[cfg(target_os = "...")]` and only run on the matching OS.

CI runs the full test suite on both `ubuntu-latest` and `macos-latest` for every pull request.

## Code style

- **Formatter** — run `cargo fmt` before committing. The CI will fail if formatting differs from the `rustfmt` defaults.
- **Linter** — run `cargo clippy` and resolve any warnings before opening a PR.
- **Error handling** — use `anyhow::Result` and `anyhow::bail!`. Avoid `unwrap()` in production code paths.
- **No async** — all I/O is synchronous; do not introduce an async runtime.
- **Dependencies** — prefer the existing crates. Add new dependencies only when absolutely necessary and discuss them in the issue or PR first.
- **Tests** — every new behaviour should be covered by a unit test. Use the `pretty_assertions` crate for clearer diff output in assertions. Mock the `Storage` trait for tests that touch the filesystem.

## Submitting changes

1. Create a feature branch from `main`:

   ```bash
   git checkout -b feat/my-improvement
   ```

2. Make your changes in small, focused commits with clear messages.

3. Ensure everything passes before pushing:

   ```bash
   cargo fmt --check
   cargo clippy
   cargo test
   ```

4. Push your branch and open a **Pull Request** against `main`.

5. Fill in the PR description explaining *what* changed and *why*.

6. Link any related issue(s) using `Closes #<issue-number>` in the PR body.

## Reporting issues

- Search [existing issues](https://github.com/iepsen/cmvm/issues) before opening a new one to avoid duplicates.
- Include your OS, architecture, cmvm version (`cmvm --version`), and the exact command you ran.
- For bugs, describe the expected and actual behaviour, and paste any error output.

## Review and merge process

- A maintainer will review your PR, usually within a few days.
- Feedback is given as review comments. Please respond to each comment or resolve it with a commit.
- Once all discussions are resolved and CI is green, a maintainer will merge the PR.
- Squash merges are preferred to keep the commit history clean.

## Resources for Rust beginners

If you are new to Rust, the following resources will help you get up to speed:

| Resource | Description |
|----------|-------------|
| [The Rust Book](https://doc.rust-lang.org/book/) | The official, beginner-friendly guide to Rust |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Learn by reading annotated example programs |
| [Rustlings](https://github.com/rust-lang/rustlings) | Small exercises to learn Rust interactively |
| [clap docs](https://docs.rs/clap/latest/clap/) | Argument parsing library used by cmvm |
| [anyhow docs](https://docs.rs/anyhow/latest/anyhow/) | Error handling library used throughout cmvm |
| [serde docs](https://serde.rs/) | Serialization/deserialization framework used for JSON |
| [reqwest docs](https://docs.rs/reqwest/latest/reqwest/) | HTTP client used for GitHub API requests |
