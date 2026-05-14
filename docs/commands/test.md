# crow test

Builds and runs the test binary.

## Usage

```bash
crow test
crow test --release
crow test --bin my_test_binary
crow test -- --test-threads=4
```

## Flags

Same as [`run`](./run.md):

| Flag | Short | What it does |
|------|-------|-------------|
| `--release` | `-r` | Run tests in release mode (faster, but harder to debug) |
| `-j <n>`, `--jobs <n>` | `-j` | Parallel jobs |
| `--bin <name>` | | Run a specific test binary |
| `<args>...` | | Extra args passed to the test runner |

## Profile

Defaults to the `test` profile (same as `dev` — no optimization, debug info, incremental).

If you want fast tests that still have debug info:

```bash
crow test               # debug, unoptimized
crow test --release     # release, optimized
```

## Passing args to tests

Use `--` to separate Crow flags from test flags:

```bash
crow test -- --test-threads=1 --nocapture
```

Everything after `--` goes to the test binary, not Crow.