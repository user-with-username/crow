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

/// Returns `true` if the two compiler kinds are compatible (belong to the same family).
fn compatible_kinds(expected: CompilerKind, detected: CompilerKind) -> bool {
    if expected == detected {
        return true;
    }

    // Clang family compatibility
    let clang_family = [
        CompilerKind::Clang,
        CompilerKind::ClangPP,
        CompilerKind::ClangCl,
    ];
    if clang_family.contains(&expected) && clang_family.contains(&detected) {
        return true;
    }

    // GCC family compatibility
    let gcc_family = [CompilerKind::Gcc, CompilerKind::Gpp];
    if gcc_family.contains(&expected) && gcc_family.contains(&detected) {
        return true;
    }

    false
}

/// Returns `true` if the compiler kind belongs to the GCC/clang family (Unix-like).
fn is_gcc_like(kind: CompilerKind) -> bool {
    matches!(
        kind,
        CompilerKind::Gcc | CompilerKind::Gpp | CompilerKind::Clang | CompilerKind::ClangPP
    )
}

/// Detects a suitable toolchain.
/// - If `preferred_compiler` is Some(path), we use that exact path and let the toolchain detect its kind.
/// - If `preferred_kind` is Some(kind), we try to find a toolchain of that kind (or compatible) and verify it.
/// - On Windows, it first attempts to find an MSVC toolchain. If that fails, it falls back to GCC.
/// - On other platforms, it uses GCC (or Clang on macOS).
pub fn detect_toolchain(
    preferred_compiler: Option<String>,
    preferred_kind: Option<CompilerKind>,
) -> Result<Box<dyn Toolchain>> {
    let preferred_kind = preferred_kind.filter(|k| *k != CompilerKind::Unknown);

    #[cfg(target_os = "windows")]
    {
        // If user explicitly wants MSVC, try that first
        if let Some(kind) = &preferred_kind {
            if *kind == CompilerKind::Msvc {
                let msvc = MsvcToolchain::detect(preferred_compiler.clone())?;
                if compatible_kinds(*kind, msvc.compiler_kind()) {
                    return Ok(Box::new(msvc));
                } else {
                    anyhow::bail!(
                        "Detected MSVC toolchain but kind mismatch (expected {:?}, got {:?})",
                        kind,
                        msvc.compiler_kind()
                    );
                }
            }
        }

        // Otherwise, try to detect any MSVC toolchain if preferred kind is not GCC-like
        if preferred_kind.as_ref().map_or(true, |k| !is_gcc_like(*k)) {
            if let Ok(msvc) = MsvcToolchain::detect(preferred_compiler.clone()) {
                if preferred_kind.map_or(true, |k| compatible_kinds(k, msvc.compiler_kind())) {
                    return Ok(Box::new(msvc));
                }
            }
        }
    }

    // Try GCC-like toolchain (GCC, Clang, ClangCl)
    if let Ok(gcc) = GccToolchain::detect(preferred_compiler) {
        if preferred_kind.map_or(true, |k| compatible_kinds(k, gcc.compiler_kind())) {
            return Ok(Box::new(gcc));
        } else {
            anyhow::bail!(
                "Detected GCC-like toolchain but kind mismatch (expected {:?}, got {:?})",
                preferred_kind.unwrap(),
                gcc.compiler_kind()
            );
        }
    }

    anyhow::bail!("Could not detect a suitable C++ toolchain")
}
