# Architecture

This document describes the internal structure of cmvm and how its parts fit together.

## Overview

cmvm is a synchronous Rust CLI application. There is no async runtime — all I/O (HTTP and filesystem) uses blocking APIs. The binary is a single self-contained executable with no daemon or background service.

## Module map

```
src/
├── main.rs        – CLI entry point: argument parsing (clap), dispatches to Commands
├── commands.rs    – High-level command implementations (install, use, list, …)
├── releases.rs    – Fetching release metadata from GitHub and managing the local cache
├── versions.rs    – Version data model: parsing, listing, and activation (symlink)
├── package.rs     – Downloading, decompressing (.tar.gz), and staging CMake archives
├── platform.rs    – Platform detection (macOS / Linux) and asset-name filtering
├── storage.rs     – Storage trait + default implementation (OS-standard directories)
├── cache.rs       – Low-level filesystem helpers (create/delete/open files and dirs)
├── http.rs        – Thin wrapper around reqwest blocking client
├── constants.rs   – Shared constants (GitHub API base URL, releases filename, …)
└── types.rs       – Shared type aliases
```

## Request flow

Below is a simplified call flow for `cmvm install <version>`:

```
main()
  └─ Commands::install_version()
        ├─ releases::build_cache()          # fetch / refresh releases.json from GitHub API
        │    └─ http::get()                 # GET https://api.github.com/repos/Kitware/CMake/releases
        ├─ releases::get_release()          # look up the requested version in the local cache
        │    └─ Version::all_from_cache()   # deserialize releases.json
        ├─ package::get_cmake_release()     # download + install the binary archive
        │    ├─ package::filter_platform_assets()   # select the correct .tar.gz for the current OS
        │    ├─ package::download()         # stream archive to cache dir
        │    ├─ package::uncompress()       # extract .tar.gz with flate2 + tar
        │    ├─ package::copy()             # copy bin/ doc/ man/ share/ to versions dir
        │    └─ package::clean()            # remove temporary archive from cache
        └─ Commands::use_version()          # update the `current` symlink
             └─ Version::use()             # std::os::unix::fs::symlink
```

## Storage layout

All files are stored under the OS-standard application directories (via the `directories` crate):

```
<data_dir>/
  versions/
    3.28.0/        # extracted CMake installation (bin/, doc/, man/, share/)
    3.27.1/
    …
  current -> versions/3.28.0   # symlink updated by `cmvm use`

<cache_dir>/
  releases.json    # merged list of all GitHub releases (refreshed in background)
```

The `Storage` trait abstracts these paths so that every command and unit test can work with a configurable root:

```rust
pub(crate) trait Storage {
    fn get_cache_dir(&self) -> Result<PathBuf>;
    fn get_data_dir(&self) -> Result<PathBuf>;
    fn get_current_version_dir(&self) -> Result<PathBuf>;
    fn get_versions_dir(&self) -> Result<PathBuf>;
}
```

Tests supply a `MockStorage` that points at a temporary directory.

## Release metadata cache

GitHub paginates its Releases API. On the first run cmvm fetches every page synchronously, saves each page as `<page>.json`, merges them into a single `releases.json`, and removes the per-page files. On subsequent runs the merge is triggered in a background thread so the CLI remains responsive.

## How to add a new platform

1. **Extend `platform.rs`** — add a new `fn <platform>_supported_definition()` that returns a `SupportedDefinition` describing the asset filename patterns and content type for the platform. Wire it into `supported_definition()`.

2. **Update `constants.rs`** — add the new OS name string to `SUPPORTED_PLATFORMS`.

3. **Test** — add `#[cfg(target_os = "...")]`-gated unit tests in `platform.rs` and `package.rs` that verify the asset filter selects the correct archive for the new platform.

4. **CI** — add the new OS to the `matrix.os` list in `.github/workflows/ci.yml` so it is automatically built and tested on every pull request.
