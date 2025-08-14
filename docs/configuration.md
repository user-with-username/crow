# Configuration

## Package Configuration

### Table of Options
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | *required* | Project identifier |
| `version` | string | *required* | Semantic version (e.g., "1.0.0") |
| `output_type` | enum | `"executable"` | `executable`, `static-lib`, `shared-lib` |
| `sources` | string[] | `["src/**/*.cpp"]` | Glob patterns for source files |
| `includes` | string[] | `[]` | Include directories |
| `libs` | string[] | `[]` | Library names (e.g., `"pthread"`) |
| `lib_dirs` | string[] | `[]` | Library search paths |

### Example
```toml
[package]
name = "image_processor"
version = "2.3.1"
output_type = "shared-lib"
sources = [
    "src/core/*.cpp",
    "src/plugins/**/*.cpp",
    "!src/plugins/experimental"
]
includes = ["include", "third_party/stb"]
libs = ["png", "jpeg", "tiff"]
lib_dirs = ["/usr/local/lib", "vendor/lib"]
```

---

## Toolchain Settings

### Table of Options
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `compiler` | string | `clang++`/`g++` | C++ compiler executable |
| `compiler_flags` | string[] | `["-std=c++17"]` | Base compiler flags |
| `linker` | string | Same as compiler | Linker executable |
| `linker_flags` | string[] | `["-lstdc++"]` | Base linker flags |
| `archiver` | string | `ar`/`lib.exe` | Static library archiver |
| `archiver_flags` | string[] | Platform-specific | Archive creation flags |

### Hooks Subtable
The `[toolchain.hooks]` subtable defines commands to run before and after the configuration resolution phase, prior to compilation and linking.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `pre_execute` | string[] | `[]` | Commands to run before configuration resolution |
| `post_execute` | string[] | `[]` | Commands to run after configuration resolution |

### Example
```toml
[toolchain]
compiler = "clang++-15"
compiler_flags = [
    "-std=c++20",
    "-fcoroutines-ts",
    "-Wall",
    "-Wextra"
]
linker = "lld"
linker_flags = [
    "-fuse-ld=lld",
    "-Wl,--as-needed"
]
archiver = "llvm-ar"

[toolchain.hooks]
pre_execute = [
    "python scripts/pre_build_setup.py",
    "echo Preparing build environment"
]
post_execute = [
    "python scripts/post_build_cleanup.py",
    "echo Build completed, cleaning up"
]
```

---

## Build Profiles

### Common Options
| Key | Type | Default (Debug) | Default (Release) |
|-----|------|-----------------|-------------------|
| `opt_level` | 0-3 | `0` | `3` |
| `defines` | string[] | `["DEBUG"]` | `["NDEBUG"]` |
| `lto` | bool | `false` | `true` |
| `flags` | string[] | `["-g"]` | `["-O3"]` |
| `incremental` | bool | `true` | `false` |

### Hooks Subtable
The `[profiles.<name>.hooks]` subtable defines commands to run before and after the configuration resolution phase for a specific profile.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `pre_execute` | string[] | `[]` | Commands to run before configuration resolution |
| `post_execute` | string[] | `[]` | Commands to run after configuration resolution |

### Example
```toml
[profiles]
[profiles.debug]
opt_level = 0
defines = [
    "DEBUG",
    "LOG_LEVEL=4",
    "SAFE_CHECKS"
]
flags = [
    "-g",
    "-fsanitize=address",
    "-D_GLIBCXX_DEBUG"
]
[profiles.debug.hooks]
pre_execute = [
    "python scripts/setup_debug_logging.py"
]
post_execute = [
    "echo Debug profile configured"
]

[profiles.release]
opt_level = 3
lto = true
incremental = false
flags = [
    "-O3",
    "-march=native",
    "-flto=thin"
]
[profiles.release.hooks]
pre_execute = [
    "echo Preparing release build"
]
post_execute = [
    "echo Release profile configured"
]

[profiles.production]
defines = [
    "NDEBUG",
    "PRODUCTION",
    "DISABLE_LOGGING"
]
[profiles.production.hooks]
pre_execute = [
    "python scripts/pre_production_setup.py"
]
post_execute = [
    "strip --strip-all ${OUTPUT}",
    "echo Production artifacts stripped"
]
```

---

## Dependencies

### Configuration Table
| Key | Type | Description |
|-----|------|-------------|
| `git` | string | Git repository URL |
| `branch` | string | Git branch/tag |
| `path` | string | Local path |
| `build.output_type` | enum | Override output type |
| `build.build_system` | enum | `crow` or `cmake` |
| `build.cmake_options` | string[] | CMake arguments |
| `build.lib_name` | string | Library name override |
| `build.pch_headers` | string[] | Precompiled headers |

### Example
```toml
[dependencies]
# Git dependency with CMake
fmt = { git = "https://github.com/fmtlib/fmt" }

# Local Crow project
core_engine = { path = "../engine/core", build = { output_type = "static-lib" }}

# Complex CMake dependency
opencv = { git = "https://github.com/opencv/opencv", build = { build_system = "cmake", cmake_options = [ "-DBUILD_TESTS=OFF", "-DWITH_QT=ON"], pch_headers = ["opencv2/core.hpp"] }}
```

---

## Target-Specific Configurations

### Matching Criteria
| Key | Description | Example Values |
|-----|-------------|----------------|
| `os` | Target OS | `windows`, `linux`, `macos` |
| `arch` | CPU architecture | `x86_64`, `arm64` |
| `os_version` | OS version | `win10`, `ubuntu22.04` |

### Overridable Settings
Any field from `[package]`, `[toolchain]`, or `[profiles]` can be overridden. Additionally, a `[targets.<name>.hooks]` subtable can specify `pre_execute` and `post_execute` commands.

### Hooks Subtable
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `pre_execute` | string[] | `[]` | Commands to run before configuration resolution |
| `post_execute` | string[] | `[]` | Commands to run after configuration resolution |

### Example
```toml
[targets.windows]
os = "windows"
compiler = "x86_64-w64-mingw32-g++"
linker_flags = ["-static", "-lws2_32"]
defines = ["WIN32_LEAN_AND_MEAN"]
[targets.windows.hooks]
pre_execute = [
    "rc.exe /fo ${PROJECT}.res ${PROJECT}.rc"
]
post_execute = [
    "echo Windows configuration completed"
]

[targets.linux_server]
os = "linux"
arch = "x86_64"
os_version = "ubuntu22.04"
compiler_flags = ["-march=x86-64-v3"]
libs = ["rt", "dl"]
[targets.linux_server.hooks]
pre_execute = [
    "echo Preparing Linux server build"
]
post_execute = [
    "echo Linux server configuration completed"
]

[targets.macos_arm]
os = "macos"
arch = "arm64"
linker_flags = ["-framework CoreFoundation"]
output_type = "shared-lib"
[targets.macos_arm.hooks]
pre_execute = [
    "python scripts/setup_macos_env.py"
]
post_execute = [
    "echo macOS ARM configuration completed"
]
```

---

## Full Configuration Example

```toml
[package]
name = "network_server"
version = "1.4.0"
output_type = "executable"
sources = ["src/**/*.cpp", "!src/legacy"]
includes = ["include", "third_party/boost"]
libs = ["ssl", "crypto", "z"]
lib_dirs = ["/usr/local/opt/openssl/lib"]

[toolchain]
compiler = "clang++"
compiler_flags = ["-std=c++20", "-fcoroutines"]
linker = "lld"

[toolchain.hooks]
pre_execute = [
    "python scripts/pre_build_setup.py",
    "echo Preparing build environment"
]
post_execute = [
    "python scripts/post_build_cleanup.py",
    "echo Build configuration completed"
]

[profiles.debug]
defines = ["DEBUG_LOGGING"]
flags = ["-g", "-fsanitize=thread"]
[profiles.debug.hooks]
pre_execute = [
    "python scripts/setup_debug_logging.py"
]
post_execute = [
    "echo Debug configuration completed"
]

[profiles.release]
lto = true
flags = ["-O3", "-march=native"]
[profiles.release.hooks]
pre_execute = [
    "echo Preparing release build"
]
post_execute = [
    "echo Release configuration completed"
]

[dependencies]
asio = { git = "https://github.com/chriskohlhoff/asio" }

[targets.windows]
compiler = "x86_64-w64-mingw32-g++"
libs = ["ws2_32", "crypt32"]
[targets.windows.hooks]
pre_execute = [
    "python scripts/setup_windows_env.py"
]
post_execute = [
    "echo Windows configuration completed"
]

[targets.arm]
arch = "arm64"
compiler_flags = ["-march=armv8-a+simd"]
[targets.arm.hooks]
pre_execute = [
    "python scripts/setup_arm_env.py"
]
post_execute = [
    "echo ARM configuration completed"
]
```