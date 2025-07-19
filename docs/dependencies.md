# Dependency Management

## Dependency Types

### Git Dependencies
```toml
[dependencies]
fmt = { git = "https://github.com/fmtlib/fmt" }
spdlog = { git = "https://github.com/gabime/spdlog", branch = "v1.x" }
```

- Cloned to `.crow/_deps` (local) or `~/.crow/_deps` (global)
- Automatically updated on subsequent builds

### Local Dependencies
```toml
[dependencies]
engine = { path = "../game_engine" }
```

- Paths relative to project root
- Copied to dependency cache when using `--global-deps`

## Build Configuration
```toml
[dependencies]
physics = { 
  git = "https://github.com/NVIDIAGameWorks/PhysX",
  build = {
    output_type = "static-lib",
    cmake_options = ["-DPHYSICS_ENABLE_TESTING=OFF"],
    lib_name = "physx"
  }
}
```

| Key | Description | Default |
|-----|-------------|---------|
| `output_type` | `static-lib`, `shared-lib` | `static-lib` |
| `build_system` | `crow`, `cmake` | Auto-detected |
| `cmake_options` | CMake arguments | `[]` |
| `lib_name` | Library name | Project name |
| `pch_headers` | Precompiled headers | `[]` |

## Build Process
1. **Git Dependencies**:
   - Cloned/updated from repository
   - Built using detected build system

2. **Local Dependencies**:
   - Copied to cache (global mode)
   - Built in-place

3. **CMake Projects**:
   - Built with appropriate generator
   - Release/Debug configurations
   - Standard dependencies disabled

4. **Crow Projects**:
   - Built recursively using Crow
   - Inherit parent profile and options

## Output
- Libraries: `<dep_root>/_crow_build/<profile>/`
- Includes: `<dep_root>/include`