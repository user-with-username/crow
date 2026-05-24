# crow.toml — The Config File

Every project has one. Here's the short version.

## Minimal example

```toml
[package]
name = "my_app"
version = "0.1.0"
standard = "20"

[dependencies]
```

That's all you need to get started.

## Full example 

```toml
[package]
name = "my_app"
version = "1.0.0"
authors = ["you <you@example.com>"]
description = "does stuff"
license = "MIT"
type = "bin"
standard = "17"

[build]
include_dirs = ["include"]
lib_dirs = ["lib"]
libs = ["m", "pthread"]
src_dirs = ["src"]
test_dirs = ["tests"]
preprocessor_defines = ["FOO=1"]
warnings_as_errors = false
parallelism = true
hooks.pre = ["echo building..."]
hooks.post = ["echo done"]
formatter = "clang-format"

[build.compiler]
kind = "gcc"
path = "/usr/bin/g++-12"
flags = ["-Wall", "-Wextra"]

[build.linker]
kind = "gcc"
path = "/usr/bin/g++-12"
flags = ["-L/custom/lib"]

[build.archiver]
kind = "gcc"

[profile.dev]
opt_level = "0"
debug = true
lto = false
incremental = true
panic = "unwind"
strip = false

[profile.release]
opt_level = "3"
debug = false
lto = true
incremental = false
panic = "unwind"
strip = true

[profile.test]
opt_level = "0"
debug = true
incremental = true

[profile.bench]
opt_level = "3"
debug = false
lto = true
strip = true

[dependencies]
boost = "1.81.0"
fmt = { git = "https://github.com/fmtlib/fmt" }
openssl = { system = true, libs = ["ssl", "crypto"] }

[workspace]
members = ["libs/*", "apps/*"]
```

## [package] — What your project is

| Field | What it does |
|-------|-------------|
| `name` | Package name. Required. |
| `version` | Semver version. Required. |
| `authors` | Your name(s). Optional. |
| `description` | One-liner. Optional. |
| `license` | "MIT", "Apache-2.0", etc. Optional. |
| `type` | `bin` (default), `exe`, `lib`, `static-lib`, `shared-lib`, `module`, `header-only`. |
| `standard` | C++ standard: `"11"`, `"14"`, `"17"`, `"20"`, `"23"`. Optional. |

### Project types

- **`bin`** or **`exe`** — executable. Looks for `src/main.cpp`.
- **`lib`** — static library (`libfoo.a` / `foo.lib`). Looks for `src/lib.cpp`.
- **`static-lib`** — same as `lib` with explicit naming.
- **`shared-lib`** — `.so` / `.dll`.
- **`module`** — C++20 module.
- **`header-only`** — no compilation, just headers.

## [build] — How to compile

| Field | Default | What it does |
|-------|---------|-------------|
| `include_dirs` | `["include"]` | Where to look for headers |
| `lib_dirs` | `[]` | Where to look for libraries |
| `libs` | `[]` | System libs to link (`-l` flags) |
| `src_dirs` | `["src"]` | Where source files live |
| `src_extensions` | `["cpp","c","cc","cxx"]` | File types to compile |
| `preprocessor_defines` | `[]` | `-D` flags |
| `warnings_as_errors` | `false` | Warnings = build failure |
| `parallelism` | `true` | Use multiple CPU cores |
| `hooks.pre` | `[]` | Commands to run before build |
| `hooks.post` | `[]` | Commands to run after build |
| `formatter` | `clang-format` | Binary of code formatter |

## [build.compiler] — Pick your compiler

Simple:
```toml
[build.compiler]
kind = "clang"
```

Detailed:
```toml
[build.compiler]
kind = "gcc"
path = "/usr/bin/g++-12"
flags = ["-Wall"]
```

Options for `kind`: `gcc`, `clang`, `msvc`, `unknown` (auto-detect).

## [build.linker] and [build.archiver]

Same format as compiler. Usually you don't need to touch these — Crow figures it out from your compiler choice.

## [profile.*] — Build profiles

See the [profiles](./profiles.md) page for the full rundown.

## [dependencies] — External libraries

See the [dependencies](./dependencies.md) page.

## [workspace] — Multi-package repos

```toml
[workspace]
members = ["libs/*", "apps/cli"]
```

See the [workspaces](./workspaces.md) page.