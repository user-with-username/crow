mod gcc;
mod msvc;

pub use gcc::GccToolchain;
pub use msvc::MsvcToolchain;

use crate::builder::kinds::{
    archiver_kind::ArchiverKind, compiler_kind::CompilerKind, linker_kind::LinkerKind,
};
use anyhowed::{Context, Result};
use std::path::PathBuf;

pub trait Toolchain: Send + Sync {
    fn compiler_kind(&self) -> CompilerKind;
    fn linker_kind(&self) -> LinkerKind;
    fn compiler_path(&self) -> &str;
    fn linker_path(&self) -> &str;
    fn archiver_path(&self) -> &str;
    fn archiver_kind(&self) -> ArchiverKind;
    fn system_include_dirs(&self) -> Vec<PathBuf>;
    fn system_library_dirs(&self) -> Vec<PathBuf>;
}

fn compatible_kinds(expected: CompilerKind, detected: CompilerKind) -> bool {
    if expected == detected {
        return true;
    }

    let clang_family = [
        CompilerKind::Clang,
        CompilerKind::ClangPP,
        CompilerKind::ClangCl,
    ];
    if clang_family.contains(&expected) && clang_family.contains(&detected) {
        return true;
    }

    let gcc_family = [CompilerKind::Gcc, CompilerKind::Gpp];
    if gcc_family.contains(&expected) && gcc_family.contains(&detected) {
        return true;
    }

    false
}

struct ToolchainType {
    name: &'static str,
    identify_compiler: fn(&str) -> Option<CompilerKind>,
    identify_linker: fn(&str) -> bool,
    is_available_on_platform: fn() -> bool,
    detect: fn(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<ArchiverKind>,
    ) -> Result<Box<dyn Toolchain>>,
}

macro_rules! register_toolchain {
    ($name:expr, $toolchain:ty, $platform_check:expr) => {
        ToolchainType {
            name: $name,
            identify_compiler: <$toolchain>::identify_compiler,
            identify_linker: <$toolchain>::identify_linker,
            is_available_on_platform: $platform_check,
            detect: |compiler_pref, linker_pref, archiver_pref, archiver_kind_pref| {
                let tc = <$toolchain>::detect(
                    compiler_pref,
                    linker_pref,
                    archiver_pref,
                    archiver_kind_pref,
                )?;
                Ok(Box::new(tc) as Box<dyn Toolchain>)
            },
        }
    };
}

const TOOLCHAIN_TYPES: &[ToolchainType] = &[
    register_toolchain!("MSVC", MsvcToolchain, || {
        #[cfg(target_os = "windows")]
        return true;
        #[cfg(not(target_os = "windows"))]
        return false;
    }),
    register_toolchain!("GCC-like", GccToolchain, || true),
];

pub fn detect_toolchain(
    preferred_compiler: Option<String>,
    preferred_compiler_kind: Option<CompilerKind>,
    preferred_linker: Option<String>,
    preferred_linker_kind: Option<LinkerKind>,
    preferred_archiver: Option<String>,
    preferred_archiver_kind: Option<ArchiverKind>,
) -> Result<Box<dyn Toolchain>> {
    let preferred_compiler_kind = preferred_compiler_kind.filter(|k| *k != CompilerKind::Unknown);
    let preferred_linker_kind = preferred_linker_kind.filter(|k| *k != LinkerKind::Unknown);
    let preferred_archiver_kind = preferred_archiver_kind.filter(|k| *k != ArchiverKind::Unknown);

    let available_types: Vec<&ToolchainType> = TOOLCHAIN_TYPES
        .iter()
        .filter(|tt| (tt.is_available_on_platform)())
        .collect();

    if available_types.is_empty() {
        anyhowed::bail!(
            "No toolchains available on this platform ({}). \
            This is unexpected - GCC-like toolchain should always be available.",
            std::env::consts::OS
        );
    }

    let mut possible_types: Vec<&ToolchainType> = Vec::new();

    for tt in &available_types {
        let compiler_matches = match &preferred_compiler {
            Some(compiler_path) => (tt.identify_compiler)(compiler_path).is_some(),
            None => true,
        };

        let linker_matches = match &preferred_linker {
            Some(linker_path) => (tt.identify_linker)(linker_path),
            None => true,
        };

        if compiler_matches && linker_matches {
            possible_types.push(*tt);
        }
    }

    if possible_types.is_empty() {
        let mut working_types = Vec::new();

        for tt in &available_types {
            match (tt.detect)(
                preferred_compiler.clone(),
                preferred_linker.clone(),
                preferred_archiver.clone(),
                preferred_archiver_kind,
            ) {
                Ok(tc) => {
                    let compiler_kind_ok = preferred_compiler_kind
                        .as_ref()
                        .map_or(true, |k| compatible_kinds(*k, tc.compiler_kind()));

                    let linker_kind_ok = preferred_linker_kind
                        .as_ref()
                        .map_or(true, |k| *k == tc.linker_kind());

                    let archiver_kind_ok = preferred_archiver_kind
                        .as_ref()
                        .map_or(true, |k| *k == tc.archiver_kind());

                    if compiler_kind_ok && linker_kind_ok && archiver_kind_ok {
                        working_types.push((tt, tc));
                    }
                }
                Err(_) => continue,
            }
        }

        if working_types.is_empty() {
            anyhowed::bail!(
                "Could not detect any suitable toolchain on {}. \
                Make sure a compatible compiler is installed and in PATH.",
                std::env::consts::OS
            );
        }

        if working_types.len() > 1 {
            eprintln!("Warning: multiple toolchains available; picking the first.");
        }

        return Ok(working_types.remove(0).1);
    }

    if possible_types.len() > 1 {
        if let Some(ref pref_kind) = preferred_compiler_kind {
            possible_types.retain(|tt| {
                if let Some(compiler_path) = &preferred_compiler {
                    if let Some(detected_kind) = (tt.identify_compiler)(compiler_path) {
                        compatible_kinds(*pref_kind, detected_kind)
                    } else {
                        if let Ok(tc) = (tt.detect)(
                            preferred_compiler.clone(),
                            preferred_linker.clone(),
                            preferred_archiver.clone(),
                            preferred_archiver_kind,
                        ) {
                            compatible_kinds(*pref_kind, tc.compiler_kind())
                        } else {
                            false
                        }
                    }
                } else {
                    if let Ok(tc) = (tt.detect)(
                        preferred_compiler.clone(),
                        preferred_linker.clone(),
                        preferred_archiver.clone(),
                        preferred_archiver_kind,
                    ) {
                        compatible_kinds(*pref_kind, tc.compiler_kind())
                    } else {
                        false
                    }
                }
            });
        }

        if possible_types.is_empty() {
            anyhowed::bail!("No toolchain matches the specified compiler kind");
        }

        if possible_types.len() > 1 {
            if let Some(ref pref_linker_kind) = preferred_linker_kind {
                possible_types.retain(|tt| {
                    if let Ok(tc) = (tt.detect)(
                        preferred_compiler.clone(),
                        preferred_linker.clone(),
                        preferred_archiver.clone(),
                        preferred_archiver_kind,
                    ) {
                        tc.linker_kind() == *pref_linker_kind
                    } else {
                        false
                    }
                });
            }
        }

        if possible_types.is_empty() {
            anyhowed::bail!("No toolchain matches the specified linker kind");
        }

        if possible_types.len() > 1 {
            let mut working_types = Vec::new();
            for tt in &possible_types {
                if let Ok(tc) = (tt.detect)(
                    preferred_compiler.clone(),
                    preferred_linker.clone(),
                    preferred_archiver.clone(),
                    preferred_archiver_kind,
                ) {
                    working_types.push(tc);
                }
            }
            return Ok(working_types.remove(0));
        }
    }

    let selected_type = possible_types[0];

    let toolchain = (selected_type.detect)(
        preferred_compiler,
        preferred_linker,
        preferred_archiver,
        preferred_archiver_kind,
    )
    .with_context(|| format!("Failed to instantiate {} toolchain", selected_type.name))?;

    Ok(toolchain)
}
