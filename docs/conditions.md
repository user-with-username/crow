# Target conditions (`cfg` expressions)

In `crow.toml`, the `[target]` table lets you override settings per platform.  
The condition inside `cfg(...)` decides when an override applies.

## Syntax

A condition can be:

- **Simple key** — `windows`, `unix`, `macos`, `linux`
- **Key = value** — `target_os = "linux"`, `target_arch = "aarch64"`
- **Composite** — `all(...)`, `any(...)`, `not(...)`
- **Raw target triple** — `"x86_64-unknown-linux-gnu"` (quoted)

### Examples

```toml
[target.'cfg(windows)'.build]
libs = ["winmm"]

[target.'cfg(target_os = "linux")'.dependencies]
alsa = "0.2"

[target.'cfg(all(windows, target_arch = "x86_64"))'.build]
libs = ["winmm", "ws2_32"]

[target.'cfg(any(macos, linux))'.dependencies]
pthread = { system = true, libs = ["pthread"] }

[target.'cfg(not(windows))'.build]
warnings_as_errors = true

[target.'"aarch64-apple-darwin"'.build.compiler]
flags = ["-mcpu=apple-m1"]
```

## Available `cfg` keys

| Key | Possible values | What it checks |
|-----|----------------|----------------|
| `windows` | (none) | `true` if OS is Windows |
| `unix` | (none) | `true` for Linux, macOS, BSDs |
| `macos` | (none) | `true` if OS is macOS |
| `linux` | (none) | `true` if OS is Linux |
| `target_os` | `"windows"`, `"linux"`, `"macos"`, `"freebsd"`, … | Operating system name |
| `target_arch` | `"x86_64"`, `"aarch64"`, `"arm"`, `"wasm32"`, … | CPU architecture |
| `target_vendor` | `"unknown"`, `"apple"`, `"pc"`, … | Vendor string |
| `target_env` | `"gnu"`, `"msvc"`, `"musl"`, … | ABI / environment |
| `target_pointer_width` | `"32"`, `"64"` | Pointer size in bits |
| `target_endian` | `"little"`, `"big"` | Byte order |
| `target_family` | `"windows"`, `"unix"` | Higher‑level family |

## Important: quoting

Because `cfg(...)` contains characters that TOML would parse as a table key, **you must quote the whole condition**:

```toml
# Correct
[target.'cfg(windows)'.build]

# Correct – triple quoted
[target.'"x86_64-unknown-linux-gnu"'.dependencies]

# Wrong – missing quotes
[target.cfg(windows).build]
```

## Notes

- Conditions are evaluated at build time, not during `crow publish`.
- Multiple matching `[target]` blocks are merged in the order they appear (later overrides earlier).
- `target_pointer_width` and `target_endian` are derived from the target triple.
- Overriding `[package]` fields is possible but not recommended.