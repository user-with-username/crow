# Crow — C++ build tool that doesn't suck

You know how C++ projects usually have a 400-line CMakeLists.txt that breaks when you look at it wrong? Crow exists so you don't have to deal with that.

One config file. One command to build. One command to run. That's the whole pitch.

## What you get

- `crow.toml` instead of CMake hell
- `crow run` — builds and runs in one shot
- Dependency management (git, registry, local paths, system libs)
- Incremental builds — only recompiles what changed
- Workspaces for multi-package repos
- Profiles: debug, release, test, bench
- Publishing your libs to a registry

## What Crow won't do (yet)

- Replace CMake for massive legacy projects
- Generate Visual Studio projects (just open the `compile_commands.json` in your IDE)
- Guarantee world peace

It's early. It works. It'll get better.