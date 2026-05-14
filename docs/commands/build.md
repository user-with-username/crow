# crow build

Compiles your project. Doesn't run it — just builds.

## Usage

```bash
crow build
crow build --release
crow build --bin server
crow build -j 4
```

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `--release` | `-r` | Build in release mode (optimized, LTO, stripped) |
| `--target <name>` | | Build for a specific target triple |
| `-j <n>`, `--jobs <n>` | `-j` | Number of parallel compilation jobs |
| `--bin <name>` | | Build only this specific binary |
| `-p <name>`, `--profile <name>` | `-p` | Use this profile: `debug`, `release`, `test`, `bench` |

## What happens

1. Reads `crow.toml`
2. Resolves and fetches dependencies (if needed)
3. Finds all `.cpp`/`.c`/`.cc`/`.cxx` files in `src_dirs`
4. Compiles each one (in parallel by default)
5. Links everything into the final binary/library
6. Saves a lockfile at `crow.lock`

## Output

```
target/<profile>/<binary_name>
```

For example: `target/debug/my_app`, `target/release/libmylib.a`

## Tips

- First build is always slow (compiles everything). Subsequent builds are fast because of incremental compilation.
- Use `-j` to match your CPU core count if you have lots of files.
- `--bin` is useful in workspaces with multiple binaries.