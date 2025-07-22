use dirs::home_dir;
use std::{env, path::PathBuf};

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
}
