# Why Crow?

You've got options for building C++. Here's why you might pick Crow over them.

## Crow vs CMake

CMake is everywhere. It's powerful. It's also... kind of a lot.

```cmake
# CMake: just to link a file
add_executable(my_app src/main.cpp)
target_include_directories(my_app PRIVATE include)
target_link_libraries(my_app PRIVATE Threads::Threads)
find_package(Threads REQUIRED)
```

```toml
# Crow: same thing
[package]
name = "my_app"
type = "bin"
```

That's it. Source files are auto-discovered in `src/`. Includes in `include/` are auto-added. No `cmake_minimum_required`, no generator selection, no out-of-source build directories.

**When CMake still wins:**
- Huge legacy codebases already on CMake
- Projects that need fine-grained control over every compiler flag per-target
- Teams that already have CMake expertise and CI pipelines built around it

**When Crow wins:**
- Starting fresh and want to get going fast
- Small to medium projects
- You don't want to think about build systems

---


## Crow vs raw Makefiles

Makefiles work. But they're manual. You're hand-writing dependency tracking, incremental build logic, cross-platform flags...

Crow does all of that automatically:
- **Incremental builds** — only recompiles what changed
- **Dependency resolution** — no more manually downloading and linking deps
- **Cross-platform** — same `crow.toml` on Linux, macOS, Windows
- **Lockfile** — reproducible builds without pinning Make versions

---

## Crow vs Just Using an IDE

IDEs are great for editing. They're terrible for sharing build config with your team.

Crow gives you a **text config file** that works the same everywhere. Your IDE can use the `compile_commands.json` Crow generates — you get the best of both.

---

## Who is Crow for?

- **Solo devs** who want to ship code, not fight build systems
- **Small teams** who need shared deps and reproducible builds
- **C++ learners** who don't want CMake as their first obstacle
- **Rust developers** doing C++ interop who want a familiar workflow

## Who should not use Crow

- Teams with massive existing CMake infrastructure (migration cost is real)
- Projects requiring exotic build steps Crow doesn't support yet (custom codegen, protobuf, etc.)
- Anyone who needs distributed remote caching at scale

---

## TL;DR

Crow is the `cargo` for C++ people who want to build things, not configure build systems. It's not for everyone — but if it fits your project, it'll save you a lot of time.