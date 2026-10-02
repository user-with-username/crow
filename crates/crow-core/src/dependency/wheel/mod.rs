use anyhowed::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod bazel;
pub mod cmake;
pub mod meson;

use crate::builder::kinds::compiler_kind::CompilerKind;

/// Result of wheel build artifacts
#[derive(Debug, Clone, Default)]
pub struct WheelArtifacts {
    /// Include paths (header generation)
    pub include_dirs: Vec<PathBuf>,
    /// Paths to compiled libraries (static/dynamic)
    pub lib_paths: Vec<PathBuf>,
    /// Library names for linking (without extension and lib prefix)
    pub lib_names: Vec<String>,
}

/// Trait for build system
pub trait Wheel: Send + Sync {
    /// Run wheel build, return artifacts
    /// compiler_flags: flags for the compiler (-std, -O, etc.)
    /// build_flags: flags for the build system itself (-D for CMake, --define for Bazel, etc.)
    /// dep_features: maps dep name → enabled features for CMake/Bazel component selection.
    /// Contains both system and registry features.
    #[allow(clippy::too_many_arguments)]
    fn build(
        &self,
        build_dir: &Path,
        profile: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
        dep_features: &HashMap<String, Vec<String>>,
    ) -> Result<WheelArtifacts>;
    /// Check if this build system is suitable for the given directory
    fn detects(&self, root: &Path) -> bool;
    /// Return root directory
    fn root(&self) -> &Path;
}

/// Wheel types (enum instead of trait object)
#[derive(Clone)]
pub enum WheelType {
    Cmake(cmake::CmakeWheel),
    Meson(meson::MesonWheel),
    Bazel(bazel::BazelWheel),
}

impl WheelType {
    pub fn new(root: &Path) -> Option<Self> {
        let cmake_wheel = cmake::CmakeWheel::new(root);
        if cmake_wheel.detects(root) {
            return Some(WheelType::Cmake(cmake_wheel));
        }
        let meson_wheel = meson::MesonWheel::new(root);
        if meson_wheel.detects(root) {
            return Some(WheelType::Meson(meson_wheel));
        }
        let bazel_wheel = bazel::BazelWheel::new(root);
        if bazel_wheel.detects(root) {
            return Some(WheelType::Bazel(bazel_wheel));
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &self,
        build_dir: &Path,
        profile: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
        dep_features: &HashMap<String, Vec<String>>,
    ) -> Result<WheelArtifacts> {
        match self {
            WheelType::Cmake(w) => w.build(
                build_dir,
                profile,
                compiler_flags,
                build_flags,
                compiler_path,
                compiler_kind,
                dep_features,
            ),
            WheelType::Meson(w) => w.build(
                build_dir,
                profile,
                compiler_flags,
                build_flags,
                compiler_path,
                compiler_kind,
                dep_features,
            ),
            WheelType::Bazel(w) => w.build(
                build_dir,
                profile,
                compiler_flags,
                build_flags,
                compiler_path,
                compiler_kind,
                dep_features,
            ),
        }
    }

    pub fn root(&self) -> &Path {
        match self {
            WheelType::Cmake(w) => w.root(),
            WheelType::Meson(w) => w.root(),
            WheelType::Bazel(w) => w.root(),
        }
    }
}

/// Create a wheel from root directory
pub fn create_wheel(root: &Path) -> Option<WheelType> {
    WheelType::new(root)
}

/// Load artifacts from a previous wheel build in the cache directory, if present.
pub fn load_wheel_cache(
    cache_build_dir: &Path,
    dep_root: &Path,
    profile: &str,
) -> Option<WheelArtifacts> {
    if !cache_build_dir.join(".wheel-built").exists() {
        return None;
    }

    let build_type = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };

    for subdir in ["cmake_wheel", "meson_wheel", "bazel_wheel"] {
        let out = cache_build_dir.join(subdir);
        if !out.is_dir() {
            continue;
        }

        let install = out.join("install");
        let mut artifacts = get_artifacts(
            dep_root,
            &out,
            vec![out.join("generated"), install.join("include")],
            vec![
                install.join("lib"),
                out.clone(),
                out.join(build_type),
                out.join(build_type.to_lowercase()),
                out.join("lib"),
            ],
            4,
        )
        .ok()?;

        // CMake File API reply is kept in the build dir, so exact include dirs
        // and library artifacts can be restored without rebuilding.
        if subdir == "cmake_wheel" {
            if let Some(exact) = cmake::CmakeWheel::load_cached(&out) {
                for dir in exact.include_dirs {
                    if !artifacts.include_dirs.contains(&dir) {
                        artifacts.include_dirs.push(dir);
                    }
                }
                // precise target artifacts beat a blind directory walk
                if !exact.lib_names.is_empty() {
                    artifacts.lib_names = exact.lib_names;
                    artifacts.lib_paths = exact.lib_paths;
                }
            }
        }

        if !artifacts.include_dirs.is_empty() || !artifacts.lib_names.is_empty() {
            return Some(artifacts);
        }
    }

    None
}

pub fn mark_wheel_built(cache_build_dir: &Path) -> Result<()> {
    std::fs::write(cache_build_dir.join(".wheel-built"), "")?;
    Ok(())
}

/// Helper function to collect artifacts from build output
pub fn get_artifacts(
    root: &Path,
    build_dir: &Path,
    additional_include_dirs: Vec<PathBuf>,
    search_dirs: Vec<PathBuf>,
    max_depth: usize,
) -> Result<WheelArtifacts> {
    let mut artifacts = WheelArtifacts::default();

    // Collect include directories
    let mut include_candidates = vec![
        root.join("include"),
        root.join("public"),
        build_dir.join("include"),
        build_dir.join("generated"),
        build_dir.join("install").join("include"),
    ];
    include_candidates.extend(additional_include_dirs);

    for dir in include_candidates {
        if dir.exists() && dir.is_dir() && !artifacts.include_dirs.contains(&dir) {
            artifacts.include_dirs.push(dir);
        }
    }

    // Define library extensions per platform.
    // MinGW/Clang on Windows use `.a` for static libraries (e.g. libfmt.a, libfmtd.a).
    #[cfg(windows)]
    let lib_extensions = ["a", "lib", "dll", "dll.a"];
    #[cfg(target_os = "macos")]
    let lib_extensions = vec!["a", "dylib", "so"];
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let lib_extensions = vec!["a", "so"];

    // Collect libraries
    for search_dir in search_dirs {
        if !search_dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&search_dir)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if should_skip_library_path(path) {
                    continue;
                }
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if lib_extensions.contains(&ext) {
                        if let Some(lib_name) = link_name_from_library_file(path) {
                            if !artifacts.lib_names.contains(&lib_name) {
                                artifacts.lib_names.push(lib_name);

                                let parent = path.parent().unwrap().to_path_buf();
                                if !artifacts.lib_paths.contains(&parent) {
                                    artifacts.lib_paths.push(parent);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(artifacts)
}

pub(crate) fn should_skip_library_path(path: &Path) -> bool {
    path.components().rev().take(3).any(|component| {
        component.as_os_str().eq_ignore_ascii_case("test")
            || component.as_os_str().eq_ignore_ascii_case("tests")
    })
}

/// Name to pass to the linker for a library file, or `None` if the file cannot
/// be linked by name.
///
/// A file such as `Luau.CLI.lib.lib` has the stem `Luau.CLI.lib`, which ends in
/// the same extension as the file itself. Once handed to the linker that name is
/// indistinguishable from a complete file name, so it is skipped.
pub(crate) fn link_name_from_library_file(path: &Path) -> Option<String> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    let stem = path.file_stem().and_then(|s| s.to_str())?;

    let stem_ext = Path::new(stem).extension().and_then(|e| e.to_str());
    if stem_ext.map_or(false, |e| e.eq_ignore_ascii_case(ext)) {
        return None;
    }

    // MSVC `.lib` files are linked by their full stem; the `lib` prefix is a
    // Unix/MinGW convention that only applies to the other extensions.
    let name = if ext.eq_ignore_ascii_case("lib") {
        stem
    } else {
        stem.strip_prefix("lib").unwrap_or(stem)
    };

    Some(name.to_string())
}