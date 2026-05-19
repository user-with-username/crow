use crate::config::macros::hooks;
use crate::config::ArchiverConfig;
use crate::config::CompilerConfig;
use crate::config::LinkerConfig;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::PathBuf;

#[derive(Debug, Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub compiler: CompilerConfig,
    pub linker: LinkerConfig,
    pub archiver: ArchiverConfig,

    #[default(vec![PathBuf::from("include")])]
    pub include_dirs: Vec<PathBuf>,

    #[default(Vec::new())]
    pub lib_dirs: Vec<PathBuf>,

    #[default(Vec::new())]
    pub libs: Vec<String>,

    #[default(vec![PathBuf::from("src")])]
    pub src_dirs: Vec<PathBuf>,

    #[default(vec![PathBuf::from("tests")])]
    pub test_dirs: Vec<PathBuf>,

    #[default(vec![PathBuf::from("benches")])]
    pub bench_dirs: Vec<PathBuf>,

    #[default(vec!["cpp".to_string(), "c".to_string(), "cc".to_string(), "cxx".to_string()])]
    pub src_extensions: Vec<String>,

    #[default(Vec::new())]
    pub preprocessor_defines: Vec<String>,

    #[serde(deserialize_with = "deserialize_hooks")]
    pub hooks: HooksConfig,

    #[default(false)]
    pub warnings_as_errors: bool,

    #[default(true)]
    pub parallelism: bool,
}

#[derive(Debug, SmartDefault, Clone)]
pub struct HooksConfig {
    #[default(Vec::new())]
    pub pre: Vec<String>,

    #[default(Vec::new())]
    pub post: Vec<String>,
}

hooks!(HooksConfig, pre, post);
