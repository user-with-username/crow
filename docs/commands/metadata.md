# crow metadata

Outputs project metadata, toolchain info, and build configuration as JSON.

## Usage

```bash
crow metadata
```

## Output Fields

| Field | Description |
|-------|-------------|
| `package` | Package name, version, type, description, authors, license |
| `workspace` | Workspace root and whether current dir is a member |
| `toolchain` | Compiler, linker, and archiver with kind, path, and flags |
| `build` | Source, include, library dirs, linked libs, test/bench dirs, parallelism |
| `paths` | Target directory location |
