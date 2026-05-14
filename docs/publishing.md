# Publishing Packages

Got a library you want to share? Publish it to the Crow registry.

## Before you publish

Make sure `crow.toml` has the basics:

```toml
[package]
name = "my-lib"
version = "1.0.0"
description = "does something useful"
license = "MIT"
repository = "https://github.com/you/my-lib"
authors = ["you <you@example.com>"]
```

Name and version are required. Everything else is recommended.

## Publish

```bash
crow publish
```

It reads your `crow.toml`, checks the version matches, and uploads.

### Custom registry

```bash
crow publish --registry https://my-registry.example.com
```

Or set `CROW_REGISTRY` env var so you don't have to pass the flag every time.

### Dry run

Checks everything without actually uploading:

```bash
crow publish --dry-run
```

### Override version on the fly

The version must match `crow.toml`, but you can override:

```bash
crow publish --version 1.0.1
```

If it doesn't match what's in `crow.toml`, Crow refuses. That's a safety net.

## Delete a published package

```bash
crow delete my-lib                        # nuke all versions
crow delete my-lib --version 1.0          # just version 1.0
crow delete my-lib -y                     # skip the "are you sure?" prompt
```

Irreversible. Always asks for confirmation unless you pass `-y`.

## Versioning

Semver, same as everywhere:

- Bugfix → bump patch: `1.0.0` → `1.0.1`
- New features → bump minor: `1.0.0` → `1.1.0`
- Breaking changes → bump major: `1.0.0` → `2.0.0`

The registry won't let you publish the same version twice.

## Default registry

```
https://github.com/user-with-username/crow-registry
```

Backed by git. You can fork it, mirror it, or run your own.