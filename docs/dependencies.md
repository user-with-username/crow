# Dependencies

You need a library that someone else wrote? Tell Crow about it in `crow.toml` under `[dependencies]`.

## Where deps come from

### From the registry (most common)

Just put a version number:

```toml
[dependencies]
boost = "1.81.0"
fmt = "^10.0"       # any 10.x
```

Crow downloads and builds it. Done.

### From git

```toml
[dependencies]
fmt = { git = "https://github.com/fmtlib/fmt" }
```

Crow clones it into `~/.crow/git_cache/` on first use, then fetches updates on subsequent builds. You can pin a version too:

```toml
fmt = { git = "https://github.com/fmtlib/fmt", version = "9.1.0" }
```

### From a local path

For when you're working on multiple packages in the same repo:

```toml
[dependencies]
my_utils = { path = "../my_utils" }
```

Nothing gets copied — Crow just builds it alongside your project and picks up changes immediately.

### System libraries (already on your machine)

```toml
[dependencies]
openssl = { system = true, libs = ["ssl", "crypto"] }
```

No downloading, no building. Just passes `-lssl -lcrypto` to the linker.

## Dev dependencies 
Dev dependencies will be used by crow if you build project with `test` or with `bench` profiles

## Build system integrations ("wheels")

Some dependencies use CMake, Meson, or Bazel. They don't have a `crow.toml` — they have a `CMakeLists.txt`, `meson.build`, or `WORKSPACE` file instead.

Crow handles this through **wheels**. When you depend on one of these projects, Crow:

1. **Detects** the build system (looks for `CMakeLists.txt`, `meson.build`, `WORKSPACE`)
2. **Builds** the project using the right tool (`cmake --build`, `meson compile`, `bazel build`)
3. **Caches** the result in `~/.crow/wheel_builds/` keyed by a hash of the source + your compiler flags
4. **Feeds back** the include directories, library paths, and library names into your build

This means any C/C++ project that uses CMake, Meson, or Bazel can be a Crow dependency — no Crow-specific changes needed on their end.

### Supported build systems

| System | Detection | Build command |
|--------|-----------|--------------|
| CMake | `CMakeLists.txt` found | `cmake --build <dir>` |
| Meson | `meson.build` found | `meson compile -C <dir>` |
| Bazel | `WORKSPACE` found | `bazel build //...` |

### How it looks from your side

It's transparent. You declare the dependency normally:

```toml
[dependencies]
protobuf = { git = "https://github.com/protocolbuffers/protobuf" }
```

Crow sees there's no `crow.toml` in the repo, finds a `CMakeLists.txt`, builds it as a wheel, and links against it. You don't write any wheel-specific config.

### Custom build flags

If you need to pass flags to the external build system:

```toml
[dependencies]
some_lib = { git = "https://...", build_flags = ["-DOPTION=ON", "-DOTHER=OFF"] }
```

These flags get forwarded to the CMake/Meson/Bazel invocation.

## Version specs

| Syntax | What it means |
|--------|---------------|
| `1.2.3` | Exactly this version |
| `^1.2.3` | >= 1.2.3 and < 2.0.0 |
| `~1.2.3` | >= 1.2.3 and < 1.3.0 |
| `>= 1.0` | At least 1.0 |
| `1.*` | Any 1.x |
| `*` | Anything |

Standard semver stuff. Same as Cargo/npm — if you've used those, you're fine.

## The lockfile

After your first build, Crow creates `crow.lock` in your project root. This pins the exact version and commit of every dependency. **Commit this file** — it makes sure everyone on your team builds with the same versions.

To get fresh deps, delete `crow.lock` and rebuild.

## Caching

Crow keeps two caches:

- `~/.crow/git_cache/` — cloned git repos (shared across projects)
- `~/.crow/wheel_builds/` — pre-built deps that use CMake/Meson/etc.

If two of your projects use the same git dep, it's only cloned once.