use std::{env::var_os, path::PathBuf};

pub struct Enviroment;

impl Enviroment {
    pub fn target_dir() -> std::path::PathBuf {
        var_os("CROW_BUILD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"))
    }
}