# crow self-update

Updates `crow` to the latest release. Downloads the correct binary for your platform and replaces the running executable.

## Usage

```bash
crow self-update
```

## Supported platforms

| OS | Architecture | Artifact |
|----|-------------|----------|
| Linux | x86_64 | `linux-x86_64` |
| Linux | aarch64 | `linux-arm64` |
| macOS | x86_64 | `macos-x86_64` |
| macOS | aarch64 | `macos-arm64` |
| Windows | x86_64 | `windows-x64.exe` |

On unsupported platforms the command exits with an error.
