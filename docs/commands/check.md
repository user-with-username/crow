# crow check

Checks your project for compilation errors without linking. Faster than `crow build` when you just want to verify that the code compiles

## Usage

```bash
crow check
crow check --release
crow check --bin server
crow check -j 4
```

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `--release` | `-r` | Check in release mode (optimized, LTO, stripped) |
| `-j <n>`, `--jobs <n>` | `-j` | Number of parallel compilation jobs |
| `--bin <name>` | | Check only this specific binary |
| `-p <name>`, `--profile <name>` | `-p` | Use this profile: `debug`, `release`, `test`, `bench` |

## What happens

1. Reads `crow.toml`
2. Resolves and fetches dependencies (if needed)
3. Finds all `.cpp`/`.c`/`.cc`/`.cxx` files in `src_dirs`
4. Compiles each one (in parallel by default)
5. **Stops before linking** — no binary is produced

## Difference from `crow build`

`crow build` compiles **and** links. `crow check` only compiles. This makes it significantly faster when you just want to verify correctness.

## Tips

- Use this in CI for fast feedback on pull requests.
- The first check is always slow (compiles everything). Subsequent checks are fast because of incremental compilation.
- Use `-j` to match your CPU core count if you have lots of files.
- `--bin` is useful in workspaces with multiple binaries.
