# crow bench

Builds and runs the benchmark binary.

## Usage

```bash
crow bench
crow bench --bin my_bench_binary
crow bench -- some_benchmark_name
```

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `-j <n>`, `--jobs <n>` | `-j` | Parallel jobs |
| `--bin <name>` | | Run a specific benchmark binary |
| `<args>...` | | Extra args passed to the benchmark |

## Profile

Uses the `bench` profile by default — max optimization, LTO, stripped. Because benchmarks need to measure optimized performance, not debug builds.

No `--release` flag needed — it's already baked in. If you want to benchmark a debug build for some reason, use `crow run -p debug --bin my_bench` instead.