use serde::Deserialize;
use std::path::PathBuf;

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
pub struct BuildConfig {
    #[serde(default)]
    pub compiler: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default = "default_include_dirs")]
    pub include_dirs: Vec<PathBuf>,
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

impl CrowConfig {
    pub fn load() -> anyhow::Result<Self> {
        use anyhow::Context;

        let content = std::fs::read_to_string("crow.toml")
            .context("could not find `crow.toml` in current directory")?;

        let config: Self = toml::from_str(&content).context("failed to parse crow.toml")?;

        Ok(config)
    }
}
