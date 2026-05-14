# crow publish

Upload your library to the Crow package registry.

## Usage

```bash
crow publish
crow publish --dry-run
crow publish --version 1.0.1
crow publish --registry https://my-registry.example.com
```

## Flags

| Flag | What it does |
|------|-------------|
| `--registry <url>` | Publish to a custom registry instead of the default |
| `--dry-run` | Do everything except actually upload — good for checking |
| `--version <ver>` | Override the version (must still match `crow.toml`) |

## Before publishing

Make sure your `crow.toml` is complete:

```toml
[package]
name = "my-lib"          # will be the crate name
version = "1.0.0"        # must match what you pass to --version
license = "MIT"          # people like to know
repository = "https://..."
description = "..."
```

## Version matching

The version you pass with `--version` has to match the `version` in `crow.toml`. If they don't match, Crow won't publish. This prevents accidental mis-publishes.

## What gets uploaded

Crow packages your `src/`, `include/`, and `crow.toml` into a git tag in the registry. Other people can then add your library to their `[dependencies]`.

## Common workflow

```bash
# 1. Make sure everything compiles
crow build --release

# 2. Do a dry run to check
crow publish --dry-run

# 3. Publish
crow publish

# 4. Someone else can now use it
# [dependencies]
# my-lib = "1.0.0"
```

## Undo

Published a version by mistake? Use [`crow delete`](./delete.md) to remove it.