# crow new

Creates a brand new Crow project from scratch.

## Usage

```bash
crow new my_project
```

This creates:

```
my_project/
├── crow.toml
├── src/
│   └── main.cpp
└── .gitignore
```

Then cd into it:

```bash
cd my_project
crow run   # build and run — prints "Hello, World!"
```

## Flags

| Flag | What it does |
|------|-------------|
| `-n`, `--no-directory` | Don't create a new folder. Sets up Crow in your current directory instead. |

## Name rules

- Must start with a letter
- Only letters, numbers, hyphens, and underscores
- Can't be empty

That's it. No magic.