# crow run

Builds your project and runs the resulting binary in one command.

## Usage

```bash
crow run
crow run --release
crow run --bin server
crow run -- some-arg -v
```

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `--release` | `-r` | Use release profile |
| `-j <n>`, `--jobs <n>` | `-j` | Parallel compilation jobs |
| `-p <name>`, `--profile <name>` | `-p` | Build profile |
| `--bin <name>` | | Run this specific binary |
| `<args>...` | | Everything after `--` goes to your program |

## How it picks which binary to run

- If there's only one binary → runs it automatically
- If there are multiple → you **must** use `--bin <name>`
- Lists available binaries in the error message if you forget `--bin`

## The `--` separator

Everything after `--` is passed directly to your executable:

```bash
crow run -- --input file.txt --verbose
#                            ^^^^^^^^^^^^^ these go to your program
```

## Exit codes

If your program exits with a non-zero code, `crow run` reports it and exits with the same code. Useful for scripting.