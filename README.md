# CMake Version Manager
cmvm is simple tool inspired by [nvm](https://github.com/nvm-sh/nvm) that manages multiple CMake versions.

## How to install
Use the following cURL command to install cmvm:
```
curl -o- https://raw.githubusercontent.com/iepsen/cmvm/master/install.sh | bash
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
