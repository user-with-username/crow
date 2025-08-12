use dirs::home_dir;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct Environment;

impl Environment {
    pub fn build_dir() -> PathBuf {
        env::var("CROW_BUILD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target"))
    }

    pub fn deps_dir(global_deps: bool) -> PathBuf {
        let base_path = if global_deps {
            home_dir().expect("Cannot get home dir").join(".crow")
        } else {
            PathBuf::from(".crow")
        };

        base_path.join("_deps")
    }

    pub fn global_deps(global_deps: bool) -> bool {
        env::var("CROW_GLOBAL_DEPS")
            .map(|val| val.eq_ignore_ascii_case("true"))
            .unwrap_or(global_deps)
    }

    pub fn quiet_mode(quiet_mode: bool) -> bool {
        env::var("CROW_QUIET_MODE")
            .map(|val| val.eq_ignore_ascii_case("true"))
            .unwrap_or(quiet_mode)
    }

    pub fn get_path_var() -> Option<OsString> {
        env::var_os("PATH")
    }

    pub fn split_path_var(path_var: &OsString) -> env::SplitPaths {
        env::split_paths(path_var)
    }

    #[cfg(windows)]
    pub fn get_pathext_var() -> Vec<OsString> {
        env::var_os("PATHEXT")
            .map(|v| {
                env::split_paths(&v)
                    .filter_map(|s| s.to_str().map(OsString::from))
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    OsString::from(".exe"),
                    OsString::from(".bat"),
                    OsString::from(".cmd"),
                ]
            })
    }

    #[cfg(not(windows))]
    pub fn get_pathext_var() -> Vec<OsString> {
        vec![OsString::from("")] // On unix exes may not have extension
    }

    pub fn is_executable(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            if let Ok(meta) = path.metadata() {
                // Check executable permission bits
                return meta.permissions().mode() & 0o111 != 0;
            }
            false
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}
