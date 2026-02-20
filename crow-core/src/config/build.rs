use crate::config::CompilerConfig;
use crate::config::LinkerConfig;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::PathBuf;

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

    #[default(vec![PathBuf::from("src")])]
    pub src_dirs: Vec<PathBuf>,

    #[default(vec!["cpp".to_string(), "c".to_string(), "cc".to_string(), "cxx".to_string()])]
    pub src_extensions: Vec<String>,

    #[default(PathBuf::from("target"))]
    pub target_dir: PathBuf,

    #[default(Vec::new())]
    pub preprocessor_defines: Vec<String>,

    #[default(true)]
    pub parallelism: bool,
}
