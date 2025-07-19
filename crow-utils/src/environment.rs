use dirs::home_dir;
use std::{
    env,
    path::{PathBuf},
};

pub struct Environment;

impl Environment {
    pub fn build_dir() -> PathBuf {
        env::var("CROW_BUILD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target"))
    }

    pub fn deps_dir(global_deps: bool) -> PathBuf {
        if global_deps {
            home_dir()
                .expect("Cannot get home dir.")
                .join(".crow/_deps")
        } else {
            PathBuf::from(".crow/_deps")
        }
    }

    pub fn parse_global_deps_env(cli_global_deps: bool) -> bool {
        match env::var("CROW_GLOBAL_DEPS") {
            Ok(val) => val.eq_ignore_ascii_case("true"),
            Err(_) => cli_global_deps,
        }
    }

    pub fn parse_quiet_mode_env(cli_quiet_mode: bool) -> bool {
        match env::var("CROW_QUIET_MODE") {
            Ok(val) => val.eq_ignore_ascii_case("true"),
            Err(_) => cli_quiet_mode,
        }
    }
}
