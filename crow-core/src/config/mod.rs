use serde::Deserialize;

mod macros;
mod linker;
mod compiler;
mod build;
pub mod profile;

pub use linker::LinkerConfig;
pub use compiler::CompilerConfig;
pub use build::BuildConfig;
pub use profile::{Profiles, DevProfile, ReleaseProfile, TestProfile, BenchProfile, Profile};

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

impl CrowConfig {
    pub fn load() -> anyhow::Result<Self> {
        use anyhow::Context;
        let content = std::fs::read_to_string("crow.toml")
            .context("could not find `crow.toml` in current directory")?;
        let config: Self = toml::from_str(&content).context("failed to parse crow.toml")?;
        Ok(config)
    }
}