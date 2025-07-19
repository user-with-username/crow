# Environment

## Build Control
| Variable | Description | Default |
|----------|-------------|---------|
| `CROW_BUILD_DIR` | Output directory | `target` |
| `CROW_GLOBAL_DEPS` | Force global dependencies | `false` |
| `CROW_QUIET_MODE` | Suppress non-critical output | `false` |

## Example Usage
```bash
# Custom build directory
export CROW_BUILD_DIR="build_output"
crow build

# Force global dependencies in CI
export CROW_GLOBAL_DEPS="true"
crow build --profile release
```
