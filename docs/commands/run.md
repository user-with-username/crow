# crow run

Builds and executes the project.

## Usage
```bash
crow run [OPTIONS]
```

## Options
| Option | Description | Default |
|--------|-------------|---------|
| `--profile <name>` | Build profile | `debug` |
| `--no-build` | Skip build phase | false |
| `--verbose` | Show build details | false |
| `--global-deps` | Use global dependencies | false |
| `--quiet` | Suppress non-critical output | false |

## Process
1. Builds project (unless `--no-build` specified)
2. Executes output binary
3. Returns program's exit code

## Examples
```bash
# Build and run debug version
crow run

# Run existing release build
crow run --profile release --no-build

# Verbose execution
crow run --verbose
```