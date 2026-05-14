# Build Profiles

Crow has four profiles. Each one tweaks compiler settings for a different situation.

## The defaults

| Profile   | Optimization | Debug info | LTO | Incremental | Strip |
|-----------|-------------|-----------|-----|-------------|-------|
| `dev`     | None        | Yes       | No  | Yes         | No    |
| `release` | Max (O3)    | No        | Yes | No          | Yes   |
| `test`    | None        | Yes       | No  | Yes         | No    |
| `bench`   | Max (O3)    | No        | Yes | No          | Yes   |

## What each one is for

- **`dev`** — Default. Fast to compile, slow to run. You're debugging, you don't need optimization.
- **`release`** — Slow to compile, fast to run. Ship this. LTO and stripping make the binary small and quick.
- **`test`** — Like dev. Tests should compile fast so you run them often.
- **`bench`** — Like release. You want accurate numbers, not debug-friendly builds.

## How to pick one

```bash
crow build                # dev (default)
crow build --release      # release
crow build -p bench       # bench
crow build -p test        # test
crow run -p release       # run with release profile
crow test --release       # run tests with release settings
```

## Customizing

Everything's in `crow.toml`:

```toml
[profile.dev]
opt_level = "1"          # a little faster, still compiles quick
incremental = true       # keep incremental on (default)

[profile.release]
opt_level = "2"          # O2 instead of O3 — nearly as fast, compiles way sooner
strip = false            # keep debug symbols for crash reports
codegen_units = 4        # more parallelism inside the compiler
```

### What you can tweak

| Option | Values | What it does |
|--------|--------|-------------|
| `opt_level` | `"0"` to `"3"`, `"s"`, `"z"` | How much to optimize |
| `debug` | `true` / `false` | Include debug info |
| `lto` | `true` / `false` / `"thin"` / `"fat"` | Link-time optimization |
| `incremental` | `true` / `false` | Cache object files between builds |
| `codegen_units` | number | Parallel LLVM units (more = faster compile, less = faster binary) |
| `panic` | `"unwind"` / `"abort"` | What to do on panic |
| `strip` | `true` / `false` | Remove debug symbols from the binary |

### LTO

- `false` — fastest compile, decent runtime
- `"thin"` — good middle ground
- `true` / `"fat"` — best optimization, takes more time and memory

### Incremental

Keeps a hash of each source file + its headers + compiler flags. If nothing changed, reuses the old object file. Big time saver during development. Lives in `target/<profile>/.fingerprint.json`.

If you get weird build errors, try nuking that file.