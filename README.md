# CMake Version Manager

[![ci](https://github.com/iepsen/cmvm/actions/workflows/ci.yml/badge.svg)](https://github.com/iepsen/cmvm/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cmvm)](https://crates.io/crates/cmvm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

cmvm is a simple CLI tool that manages multiple CMake versions on macOS and Linux. Switch between versions instantly without manual downloads or path juggling.

## Why cmvm?

- **Multiple versions side-by-side** — install as many CMake versions as you need and switch between them with a single command.
- **No admin rights required** — everything is stored under your home directory.
- **Fast** — release metadata is cached locally so most operations are instant.
- **Simple** — a handful of intuitive sub-commands, nothing more.

## Supported platforms

| Platform | Architecture |
|----------|-------------|
| macOS    | x86_64, arm64 (universal) |
| Linux    | x86_64 |

## How to install

### Homebrew
```
brew tap iepsen/cmvm
brew install cmvm
```

### Cargo
```
cargo install cmvm
```

### Pre-built binaries
Pre-built binaries are [available for download](https://github.com/iepsen/cmvm/releases) for both macOS and Linux since [v0.3.3](https://github.com/iepsen/cmvm/releases/tag/v0.3.3).

## Adding cmake to the path
Once cmvm is installed, add the `current` symlink to your `$PATH` so the shell can find the active CMake binary. Run the following command for the exact export line to add to your shell profile:

```
cmvm shell
```

Add the printed line to your `~/.bashrc`, `~/.zshrc`, or equivalent shell configuration file.

## Usage

### Install a CMake version

```
cmvm install 3.28.0
```

If the version is already installed, cmvm switches to it immediately.

### Switch to a CMake version

```
cmvm use 3.28.0
```

### List installed versions

```
cmvm list
```

The active version is marked with `*`.

### List available versions to install

```
cmvm list-remote
```

Fetches the list from GitHub Releases (cached locally after the first run).

### Uninstall a CMake version

```
cmvm uninstall 3.28.0
```

### Show shell PATH instructions

```
cmvm shell
```

### Show all commands and options

```
cmvm help
```

## How to contribute
Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

Feel free to [open an issue](https://github.com/iepsen/cmvm/issues) for bug reports, feature requests, or questions.
