# crow clean

Deletes the `target/` directory. That's it.

## Usage

```bash
crow clean
```

## When to use it

- Build is acting weird and you want a fresh start
- You just want to free up disk space
- Incremental cache is stale and rebooting doesn't help

## What it removes

Everything in `target/`:

```
target/
├── debug/          # all debug build artifacts
├── release/        # all release build artifacts
├── test/           # test artifacts
└── bench/          # benchmark artifacts
```

After `crow clean`, the next `crow build` will recompile everything from scratch. This includes re-resolving and re-downloading dependencies (though cached git repos and wheels in `~/.crow/` are kept).