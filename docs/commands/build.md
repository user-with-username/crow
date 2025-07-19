# crow build

Compiles the current project.

## Usage
```bash
crow build [OPTIONS]
```

## Options
| Option | Description | Default |
|--------|-------------|---------|
| `--profile <name>` | Build profile | `debug` |
| `--jobs <N>` | Parallel jobs | CPU cores |
| `--verbose` | Show detailed output | false |
| `--global-deps` | Use global dependencies | false |
| `--quiet` | Suppress non-critical output | false |

## Environment Variables
| Variable | Description |
|----------|-------------|
| `CROW_BUILD_DIR` | If set, overrides the default output directory and builds to the specified path |

## Process
1. Loads configuration from `crow.toml`
2. Resolves dependencies
3. Compiles source files in parallel
4. Links final artifact
5. Stores build cache

## Output
- Default location: `target/<profile>/`
- If `CROW_BUILD_DIR` is set: `<CROW_BUILD_DIR>/<profile>/`
- Executables: `<output_dir>/<project_name>`
- Libraries: `<output_dir>/lib<name>.a|so|dylib|lib`
- Object files: `<output_dir>/`

## Examples
```bash
# Default debug build
crow build

# Release build with 8 jobs
crow build --profile release --jobs 8

# Verbose build with global dependencies
crow build --global-deps --verbose

# Build to custom directory
export CROW_BUILD_DIR="/mypath/smt/"
crow build
```