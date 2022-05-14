# CMake Version Manager

[![ci](https://github.com/iepsen/cmvm/actions/workflows/ci.yml/badge.svg)](https://github.com/iepsen/cmvm/actions/workflows/ci.yml) ![Crates.io](https://img.shields.io/crates/v/cmvm)

cmvm is simple tool that manages multiple CMake versions.

## How to install

As cmvm is built in rust, the only way to install cmvm is to get it from cargo for now. Follow [this instruction](https://doc.rust-lang.org/cargo/getting-started/installation.html) to get it.


With cargo in place you can install cmvm by running the following:
```
cargo install cmvm
```

The next step is to add cmake current version on your path. Use the following command to get instructions:
```
cmvm shell
```

## Usage

Install a CMake version:

```
cmvm install 3.20.1
```

List available cmake versions to install:

```
cmvm list-remote
```

List cmake versions managed by cmvm installed:

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