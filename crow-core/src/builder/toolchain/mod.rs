//! Toolchain detection and management for C/C++ compilers.

mod gcc;
mod msvc;

pub use gcc::GccToolchain;
pub use msvc::MsvcToolchain;

use crate::builder::kinds::{compiler_kind::CompilerKind, linker_kind::LinkerKind};
use anyhow::Result;
use std::path::PathBuf;

/// A trait representing a complete compilation toolchain.
pub trait Toolchain: Send + Sync {
    /// Returns the compiler kind (Gcc, Gpp, Clang, ClangPP, Msvc).
    fn compiler_kind(&self) -> CompilerKind;

    /// Returns the linker kind (Ld, Lld, Link, etc.).
    fn linker_kind(&self) -> LinkerKind;

    /// Returns the path to the compiler executable.
    fn compiler_path(&self) -> &str;

    /// Returns the path to the linker executable.
    fn linker_path(&self) -> &str;

    /// Returns the system include directories that should be passed to the compiler.
    fn system_include_dirs(&self) -> Vec<PathBuf>;

    /// Returns the system library directories that should be passed to the linker.
    fn system_library_dirs(&self) -> Vec<PathBuf>;
}

/// Detects a suitable toolchain.
/// - If `preferred_compiler` is Some(path), we use that exact path and let the toolchain detect its kind.
/// - If `preferred_kind` is Some(kind), we try to find a toolchain of that kind.
/// - On Windows, it first attempts to find an MSVC toolchain. If that fails, it falls back to GCC.
/// - On other platforms, it uses GCC (or Clang on macOS).
pub fn detect_toolchain(
    preferred_compiler: Option<String>,
    preferred_kind: Option<CompilerKind>,
) -> Result<Box<dyn Toolchain>> {
    let kind = preferred_kind.unwrap_or(CompilerKind::Unknown);

    #[cfg(target_os = "windows")]
    if !matches!(kind, CompilerKind::Gcc | CompilerKind::Gpp | CompilerKind::Clang | CompilerKind::ClangPP) {
        if let Ok(msvc) = MsvcToolchain::detect(preferred_compiler.clone()) {
            return Ok(Box::new(msvc));
        }
    }

    if let Ok(gcc) = GccToolchain::detect(preferred_compiler) {
        return Ok(Box::new(gcc));
    }

    anyhow::bail!("Could not detect a suitable C++ toolchain")
}