# CRow

**Cargo, but for C/C++**

![CRow Logo](assets/crow_logo.png)

> [!IMPORTANT]
> CRow is in the very early alpha testing. Not all features are stable

---

## Overview

CRow is a no-nonsense build system for C++ designed for developers who want:
- **Simple configuration** (just `crow.toml`)
- **Fast builds**
- **Built-in dependency manager**

---

## Quick Start

### Install
- **Pre-built binaries**: Download from [Releases](https://github.com/base-of-base/crow/releases)
- **Build from source**:
```bash
git clone https://github.com/base-of-base/crow
cd crow
cargo install --path .
```

### Create a project
```bash
crow init my_project && cd my_project
```

### Run
```bash
crow --profile release
```

---

## Community

We welcome contributions! Please read our:
- [Contributor Guidelines](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## License

CRow is distributed under the [MIT License](LICENSE).

---

> *"Simplicity is prerequisite for reliability."* — **Edsger W. Dijkstra**