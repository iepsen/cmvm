# CMake Version Manager

[![ci](https://github.com/iepsen/cmvm/actions/workflows/ci.yml/badge.svg)](https://github.com/iepsen/cmvm/actions/workflows/ci.yml) ![Crates.io](https://img.shields.io/crates/v/cmvm)

cmvm is simple tool that manages multiple CMake versions for macOS and Linux platforms.

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

## Adding cmake to the path
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

## How to contribute
You can contribute on this project by
- [Opening  an issue](https://github.com/iepsen/cmvm/issues) if you find a bug
- Being proactive and open a pull request to fix an issue or suggest improvement
- Starting a [disucssion](https://github.com/iepsen/cmvm/discussions) if you have a specific topic to discuss
