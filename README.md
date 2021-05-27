# CMake Version Manager

cmvm is simple tool inspired by [nvm](https://github.com/nvm-sh/nvm) that manages multiple CMake versions.

## How to install

Tap the CMake Version Manager [Homebrew](https://brew.sh/) formulae and install it:

```
brew tap iepsen/cmvm
brew install cmvm
```

## Usage

Install a CMake version:

```
cmvm install 3.20.1
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

## Support
Only macOS is supported for now.
