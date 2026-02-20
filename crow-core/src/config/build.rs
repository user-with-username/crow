use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::PathBuf;
use crate::config::CompilerConfig;
use crate::config::LinkerConfig;

#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub compiler: CompilerConfig,
    pub linker: LinkerConfig,
    #[default(vec![PathBuf::from("include")])]
    pub include_dirs: Vec<PathBuf>,
    #[default(Vec::new())]
    pub lib_dirs: Vec<PathBuf>,
    #[default(Vec::new())]
    pub libs: Vec<String>,
}