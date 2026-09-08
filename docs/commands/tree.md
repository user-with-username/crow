# crow tree

Displays the dependency tree for the project, similar to `cargo tree`.

## Usage

```bash
crow tree
crow tree --release
crow tree -p release
crow tree --depth 2
crow tree --no-dedupe
```

## Flags

| Flag | Short | What it does |
|------|-------|-------------|
| `--release` | `-r` | Show the dependency tree for the release profile |
| `--target <name>` | | Filter for a specific target platform |
| `-p <name>`, `--profile <name>` | `-p` | Use this profile (default: `debug`) |
| `-D <n>`, `--depth <n>` | `-D` | Maximum display depth of the dependency tree |
| `--no-dedupe` | | Do not de-duplicate; repeated dependencies are shown in full |

## Output

The tree is printed using Unicode box-drawing characters:

```
root v1.0.0
├── dep1 v2.0.0
│   ├── dep2 v3.0.0
│   └── dep3 v1.5.0
└── dep4 v4.0.0
    └── dep2 v3.0.0 (*)
```

- Package names are shown in **bold**
- Versions are shown in *dim*
- `(*)` marks a package that appears more than once in the tree (deduplicated by default)
- The `│` and `├`/`└` characters show the tree structure

## Deduplication

By default, when a dependency appears multiple times in the tree (because multiple packages depend on it), only the first occurrence is shown in full with its subtree. Subsequent occurrences are marked with `(*)` and their children are not shown again. Use `--no-dedupe` to show every occurrence.

## Depth

Use `--depth` to limit how deep the tree is displayed. For example, `--depth 1` shows only direct dependencies without their children.
