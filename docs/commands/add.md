# crow add

Adds a dependency to your project.

## Usage

```bash
crow add fmt
crow add fmt --version 1.0
crow add fmt --registry https://github.com/user-with-username/crow-registry
crow add --dev fmt 
```

This adds the package to `crow.toml`:

```toml
[dependencies]
fmt = "1.0"
```

## Arguments

| Argument | What it does |
|----------|-------------|
| `<package>` | Name of the package to add |

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `--version <ver>` | `-v` | Version requirement (e.g., `1.0`, `^2.0`, `>=1.0`) |
| `--registry <url>` | | Override the registry URL |
| `--dev` | | Adds dependency to the `dev-dependencies` section |

## How it works

1. Resolves the package name against the registry
2. Finds the latest compatible version (or uses the one you specified)
3. Adds it to `[dependencies]` or `dev-dependencies` in `crow.toml`

If the package is already in your dependencies, Crow will tell you and do nothing.

## Version specifiers

- No version → uses latest compatible (`*`)
- `1.0` → exact version
- `^1.0` → compatible with 1.x.x
- `~1.0` → compatible with 1.0.x
- `>=1.0` → at least 1.0