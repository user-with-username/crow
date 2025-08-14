use super::*;
use crow_utils::logger::{LogLevel, Logger};
use crow_utils::Environment;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub trait FlagsConverter {
    fn find_executable_in_path(name: &str) -> Option<PathBuf>;
    fn resolve_compiler(compiler: &str) -> Option<PathBuf>;
    fn detect_compiler_flavour(compiler_path: &Path, logger: &Logger) -> CompilerFlavor;
    fn convert_args_for_msvc(args: &[OsString], output: &Path) -> Vec<OsString>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompilerFlavor {
    GnuLike,  // GCC Clang
    MsvcLike, // MSVC clang-cl
}

impl FlagsConverter for BuildSystem {
    fn find_executable_in_path(name: &str) -> Option<PathBuf> {
        if name.contains(std::path::MAIN_SEPARATOR) {
            let p = PathBuf::from(name);
            return if Environment::is_executable(&p) {
                Some(p)
            } else {
                None
            };
        }

        let path_var = Environment::get_path_var()?;
        let paths = Environment::split_path_var(&path_var);

        let exts: Vec<OsString> = Environment::get_pathext_var();

        for dir in paths {
            for ext in &exts {
                let candidate = dir.join(format!("{}{}", name, ext.to_string_lossy()));
                if Environment::is_executable(&candidate) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn resolve_compiler(compiler: &str) -> Option<PathBuf> {
        if compiler.contains(std::path::MAIN_SEPARATOR) {
            let p = PathBuf::from(compiler);
            if Environment::is_executable(&p) {
                return Some(p);
            }
        }

        if let Some(found) = BuildSystem::find_executable_in_path(compiler) {
            return Some(found);
        }
        None
    }

    fn detect_compiler_flavour(compiler_path: &Path, logger: &Logger) -> CompilerFlavor {
        if logger.verbose {
            logger.log(LogLevel::Dim, "Determining compiler flavor by file name", 2);
        }
        if let Some(fname_os) = compiler_path.file_name() {
            if let Some(fname) = fname_os.to_str() {
                let f = fname.to_lowercase();

                if f.contains("clang-cl") || f == "cl.exe" || f == "clang++" {
                    logger.log(
                        LogLevel::Dim,
                        &format!("Detected MSVC-like flavor ('{}')", f),
                        2,
                    );
                    return CompilerFlavor::MsvcLike;
                }
                if f.contains("clang")
                    || f.contains("g++")
                    || f.contains("gcc")
                    || f.contains("cc")
                    || f.contains("c++")
                {
                    logger.log(
                        LogLevel::Dim,
                        &format!("Detected GNU-like flavor ('{}')", f),
                        2,
                    );
                    return CompilerFlavor::GnuLike;
                }
            }
        }

        logger.log(
            LogLevel::Warn,
            &format!(
                "Could not determine compiler type for '{}'. Defaulting to GNU-like",
                compiler_path.display()
            ),
            1,
        );
        CompilerFlavor::GnuLike
    }

    fn convert_args_for_msvc(args: &[OsString], output: &Path) -> Vec<OsString> {
        let mut out: Vec<OsString> = Vec::new();
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let s = arg.to_string_lossy();
            if s == "-c" {
                out.push(OsString::from("/c"));
            } else if s == "-o" {
                if let Some(next) = it.next() {
                    let fo = format!("/Fo{}", next.to_string_lossy());
                    out.push(OsString::from(fo));
                }
            } else if s.starts_with("-o") && s.len() > 2 {
                let path_part = &s[2..];
                let fo = format!("/Fo{}", path_part);
                out.push(OsString::from(fo));
            } else if s.starts_with("-I") && s.len() > 2 {
                let include = &s[2..];
                out.push(OsString::from(format!("/I{}", include)));
            } else if s == "-I" {
                if let Some(next) = it.next() {
                    out.push(OsString::from(format!("/I{}", next.to_string_lossy())));
                }
            } else if s.starts_with("-D") && s.len() > 2 {
                let def = &s[2..];
                out.push(OsString::from(format!("/D{}", def)));
            } else if s == "-std=c++17"
                || s == "-std=gnu++17"
                || s == "-std=gnu++2a"
                || s == "-std=c++2a"
            {
                out.push(OsString::from("/std:c++17"));
            } else if s.starts_with("-O") && s.len() >= 2 {
                let lvl = &s[2..];
                match lvl {
                    "0" => out.push(OsString::from("/Od")),
                    "1" => out.push(OsString::from("/O1")),
                    "2" | "3" | "s" | "z" => out.push(OsString::from("/O2")),
                    _ => {
                        out.push(OsString::from("/O2"));
                    }
                }
            } else if s == "-g" {
                out.push(OsString::from("/Zi"));
            } else if s == "-MMD" || s == "-MF" {
                if s == "-MF" {
                    let _ = it.next();
                }
            } else if s == "-flto" {
                out.push(OsString::from("/GL"));
                out.push(OsString::from("-fuse-ld=lld"));
            } else {
                out.push(arg.clone());
            }
        }

        let has_fo = out.iter().any(|a| a.to_string_lossy().starts_with("/Fo"));
        if !has_fo {
            out.push(OsString::from(format!("/Fo{}", output.to_string_lossy())));
        }

        let has_c = out.iter().any(|a| a.to_string_lossy() == "/c");
        if !has_c {
            out.push(OsString::from("/c"));
        }

        out.push(OsString::from("/EHsc"));
        out
    }
}
