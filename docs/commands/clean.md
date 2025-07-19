# crow clean

Removes build artifacts.

## Usage
```bash
crow clean [OPTIONS]
```

## Options
| Option | Description | Default |
|--------|-------------|---------|
| `--all` | Remove dependencies too | false |
| `--quiet` | Suppress output | false |

## Process
1. Deletes build directory (`target/`)
2. With `--all`: Also removes `.crow/_deps`
3. Preserves global dependencies (`~/.crow/_deps`)

## Examples
```bash
# Clean build artifacts
crow clean

# Clean everything including dependencies
crow clean --all
```