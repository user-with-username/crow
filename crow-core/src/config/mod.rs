use serde::Deserialize;
use std::path::PathBuf;

use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;

#[derive(Deserialize)]
pub struct CrowConfig {
    pub package: Package,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub profile: Profiles,
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct LinkerConfig {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub kind: Option<LinkerKind>,
}

#[derive(Deserialize, Default, Clone)]
pub struct CompilerConfig {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub kind: Option<CompilerKind>,
}

#[derive(Deserialize, Default, Clone)]
pub struct BuildConfig {
    #[serde(default)]
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub linker: LinkerConfig,
    #[serde(default)]
    pub include_dirs: Option<Vec<PathBuf>>,  // Опционально
    #[serde(default)]
    pub lib_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub libs: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct Profiles {
    #[serde(default)]
    pub dev: Profile,
    #[serde(default)]
    pub release: Profile,
}

#[derive(Deserialize, Default, Clone)]
pub struct Profile {
    #[serde(default = "default_opt_level")]
    pub opt_level: String,
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default)]
    pub lto: bool,
}

fn default_opt_level() -> String {
    "0".to_string()
}

fn default_debug() -> bool {
    true
}

fn default_include_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from("include")]
}

impl BuildConfig {
    pub fn get_include_dirs(&self) -> Vec<PathBuf> {
        self.include_dirs.clone().unwrap_or_else(default_include_dirs)
    }
}

impl CrowConfig {
    pub fn load() -> anyhow::Result<Self> {
        use anyhow::Context;

        let content = std::fs::read_to_string("crow.toml")
            .context("could not find `crow.toml` in current directory")?;

        let config: Self = toml::from_str(&content).context("failed to parse crow.toml")?;

        Ok(config)
    }
}