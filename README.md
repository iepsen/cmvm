# cmvm
cmvm is a CMake version manager inspired by [nvm](https://github.com/nvm-sh/nvm). It gives you the ability to have multiple CMake versions and switch between them.

## How to install
The easiest way to install cmvm is by running the following in your terminal:
```
curl -o- https://raw.githubusercontent.com/iepsen/cmvm/master/install.sh | bash
```

## Usage
cmvm without any parameters will print a help message to show the commands available:
```
➜  ~ cmvm

Usage

 cmvm list               List all cmake versions installed
 cmvm install <version>  Install a cmake version (x.y.z format)
 cmvm use <version>      Use a cmake version (x.y.z format)
 cmvm help               Print this help message

Example
 cmvm install 3.20.1     Install cmake version 3.20.1
 cmvm use 3.19.8         Set cmake version 3.19.8 as current
```

## Support
Only macOS is supported for now.
