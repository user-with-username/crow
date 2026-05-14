# crow delete

Remove a package from the Crow registry. Irreversible.

## Usage

```bash
crow delete my-package
crow delete my-package --version 1.0    # delete just one version
crow delete my-package -y               # skip confirmation
```

## Flags

| Flag | What it does |
|------|-------------|
| `<package>` | **Required.** Name of the package to delete. |
| `--version <ver>` | Delete only this specific version. Without it, deletes **all** versions. |
| `--registry <url>` | Use a custom registry instead of the default. |
| `-y`, `--yes` | Skip the "are you sure?" confirmation. |
| `--dry-run` | Show what would happen without actually deleting. |

## Confirmation

Crow will always ask you to confirm before deleting (unless you pass `-y`):

```
This action cannot be undone.
Delete all versions of `my-package`? [y/N]
```

## Example: delete one version

```bash
crow delete my-lib --version 0.1.0
```

This removes only v0.1.0. Other versions stay available.

## Example: nuke everything

```bash
crow delete my-lib -y
```

Deletes every version of `my-lib`. No going back.