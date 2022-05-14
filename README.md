# CMake Version Manager

[![ci](https://github.com/iepsen/cmvm/actions/workflows/ci.yml/badge.svg)](https://github.com/iepsen/cmvm/actions/workflows/ci.yml) ![Crates.io](https://img.shields.io/crates/v/cmvm)

cmvm is simple tool that manages multiple CMake versions.

## How to install

### Homebrew
```
brew tap iepsen/cmvm
brew install cmvm
```

### Cargo (Rust developers)
```
cargo install cmvm
```

### Releases
Binaries are [available for download](https://github.com/iepsen/cmvm/releases) on both macOS and Linux platforms since [v0.3.3](https://github.com/iepsen/cmvm/releases/tag/v0.3.3). 

## Path instructions
Once you have cmvm installed, you need to add CMake current version on your path. Use the following command to get instructions:
```
cmvm shell
```

## Usage

Install a CMake version:

```
cmvm install 3.20.1
```

List available CMake versions to install:

```
cmvm list-remote
```

List CMake versions managed by cmvm installed:

```
cmvm list
```

Switch to a CMake version:

```
cmvm use 3.20.1
```

List all commands available and usage examples:

```
cmvm help
```

## Supported platforms
It's expected to cmvm to work on macOS and Linux platforms. Found a bug? Please, file a ticket or open a PR to fix it.