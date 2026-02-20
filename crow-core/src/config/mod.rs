use serde::Deserialize;

mod build;
mod compiler;
mod linker;
mod macros;
pub mod profile;
mod r#type;

pub use build::BuildConfig;
pub use compiler::CompilerConfig;
pub use linker::LinkerConfig;
pub use profile::{BenchProfile, DevProfile, Profile, Profiles, ReleaseProfile, TestProfile};
pub use r#type::{BinaryConfig, LibraryConfig, ProjectType, TargetType};

#[derive(Deserialize)]
pub struct CrowConfig {
    pub package: Package,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub profile: Profiles,
    #[serde(default)]
    pub r#type: ProjectType,
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
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
