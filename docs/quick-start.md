# Quick Start

## Make a project

```bash
crow new my_project
cd my_project
crow run
```

That's it. You just built and ran a C++ project.

This creates:

```
my_project/
├── crow.toml          # your config
├── src/
│   └── main.cpp       # hello world starter
└── .gitignore
```

## Just build, don't run

```bash
crow build
```

Output lands in `target/debug/`.

## Release mode

```bash
crow build --release
crow run --release
```

## Pass args to your program

Everything after `--` goes to your binary:

```bash
crow run -- --verbose --input data.txt
```

## Init in an existing folder

Already have source files? Run this in the folder:

```bash
crow init
```

It'll create a `crow.toml` and a basic `src/main.cpp`. If files already exist it'll ask before overwriting — use `--quiet` to skip the prompt.

## What needs to be in your project

At minimum: a `crow.toml` with a `[package]` section, and at least one `.cpp` file in `src/`. Crow walks the `src/` directory and compiles everything it finds with `.cpp`, `.c`, `.cc`, or `.cxx` extensions.