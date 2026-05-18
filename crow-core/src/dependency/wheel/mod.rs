use anyhow::Result;
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
    fn build(
        &self,
        build_dir: &Path,
        profile: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
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
        // Create temporary instances for checking
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

    pub fn build(
        &self,
        build_dir: &Path,
        profile: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
    ) -> Result<WheelArtifacts> {
        match self {
            WheelType::Cmake(w) => {
                w.build(build_dir, profile, compiler_flags, build_flags, compiler_path, compiler_kind)
            }
            WheelType::Meson(w) => {
                w.build(build_dir, profile, compiler_flags, build_flags, compiler_path, compiler_kind)
            }
            WheelType::Bazel(w) => {
                w.build(build_dir, profile, compiler_flags, build_flags, compiler_path, compiler_kind)
            }
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
    let lib_extensions = vec!["a", "lib", "dll", "dll.a"];
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
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let lib_name = stem.trim_start_matches("lib").to_string();

                            if !artifacts.lib_names.contains(&lib_name) {
                                artifacts.lib_names.push(lib_name.clone());

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
    path.components().any(|component| {
        component
            .as_os_str()
            .eq_ignore_ascii_case("test")
            || component
                .as_os_str()
                .eq_ignore_ascii_case("tests")
    })
}

pub(crate) fn link_name_from_library_file(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|stem| stem.trim_start_matches("lib").to_string())
}
