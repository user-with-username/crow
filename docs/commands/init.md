# crow init

Turns an existing directory into a Crow project. Think `cargo init` for C++.

## Usage

```bash
cd /path/to/existing/project
crow init
```

This creates:

- `crow.toml` — derives the package name from your directory name
- `src/main.cpp` — a basic hello-world starter

If those files already exist, Crow will **ask before overwriting**.

## Flags

| Flag | What it does |
|------|-------------|
| `-q`, `--quiet` | Overwrite without asking. No confirmation. |

## What you get

The generated `crow.toml`:

```toml
[package]
name = "your_directory_name"
version = "0.1.0"
standard = "20"

[dependencies]
```

The generated `src/main.cpp`:

```cpp
#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
```

## Use case

You've been working on a C++ project with raw Makefiles or no build system, and you want Crow to manage it. Run `crow init` and you're set. You'll probably want to tweak the `crow.toml` afterwards (add deps, change the type, etc.).