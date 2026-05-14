# Workspaces

A workspace lets you have multiple Crow packages in one repo. One `crow build` builds them all.

## Set it up

Create a root `crow.toml` with a `[workspace]` section and **no** `[package]`:

```toml
# crow.toml at repo root

[workspace]
members = [
    "apps/cli",
    "apps/server",
    "libs/utils",
]
```

Each member gets its own `crow.toml`:

```
my_workspace/
├── crow.toml              # workspace root (no [package] here)
├── apps/
│   ├── cli/crow.toml
│   └── cli/src/main.cpp
├── apps/
│   ├── server/crow.toml
│   └── server/src/main.cpp
└── libs/
    └── utils/
        ├── crow.toml
        └── src/lib.cpp
```

## Build everything

```bash
crow build          # all members, debug
crow build --release
```

## Build just one

```bash
crow build --bin cli
crow build --bin server
```

## Run from a workspace

If there's one binary, `crow run` picks it up. Multiple? Specify which:

```bash
crow run --bin cli
crow run --bin server
```

## Test everything

```bash
crow test               # all members
crow test --bin cli     # just one
```

## Members can depend on each other

```toml
# apps/cli/crow.toml
[dependencies]
utils = { path = "../../libs/utils" }
```

Changes are picked up immediately — no publishing needed.

## Wildcard members

```toml
[workspace]
members = ["apps/*", "libs/*"]
```

Every subdirectory with a `crow.toml` gets included.

## One lockfile

`crow.lock` sits at the workspace root and covers all members. Commit it.

## Gotchas

- All members share the same profile in a single build command. Need different profiles? Run `crow build` from each member's directory separately.
- No `exclude` support yet — if you don't want a directory in the workspace, don't list it in `members`.