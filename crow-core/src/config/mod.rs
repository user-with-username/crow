use serde::Deserialize;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

mod build;
mod compiler;
mod linker;
mod macros;
pub mod profile;
pub mod r#type;
mod workspace;
mod package;
mod archiver;
pub use archiver::ArchiverConfig;


pub use build::BuildConfig;
pub use compiler::CompilerConfig;
pub use linker::LinkerConfig;
pub use profile::{BenchProfile, DevProfile, Profile, Profiles, ReleaseProfile, TestProfile};
pub use r#type::{BinaryConfig, LibraryConfig, ProjectType, TargetType};
pub use workspace::Workspace;
pub use package::Package;

#[derive(Deserialize, Debug, Clone)]
pub struct CrowConfig {
    /// Package definition. Optional for virtual manifests (workspaces).
    pub package: Option<Package>,

    #[serde(default)]
    pub workspace: Option<workspace::WorkspaceConfig>,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub profile: Profiles,

    #[serde(default)]
    pub r#type: ProjectType,
}

impl CrowConfig {
    /// Loads configuration by searching for `crow.toml` upwards from the current directory.
    /// Returns the config and the path to the directory containing it.
    pub fn load() -> Result<(Self, PathBuf)> {
        let current_dir = std::env::current_dir().context("failed to get current directory")?;
        Self::find_in_tree(&current_dir)
    }

    /// Loads configuration from a specific file path.
    pub fn load_from(path: &Path) -> Result<(Self, PathBuf)> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        let parent = path
            .parent()
            .context("failed to get parent directory of config file")?
            .to_path_buf();

        Ok((config, parent))
    }

    /// Recursively searches upwards for `crow.toml`.
    fn find_in_tree(start_path: &Path) -> Result<(Self, PathBuf)> {
        let mut check_path = start_path.to_path_buf();

        loop {
            let config_path = check_path.join("crow.toml");
            if config_path.exists() {
                return Self::load_from(&config_path);
            }

            if let Some(parent) = check_path.parent() {
                check_path = parent.to_path_buf();
            } else {
                break;
            }
        }

        anyhow::bail!(
            "could not find `crow.toml` in {} or any parent directory",
            start_path.display()
        );
    }

    /// Checks if this is a virtual manifest (workspace without a root package).
    pub fn is_virtual(&self) -> bool {
        self.package.is_none() && self.workspace.is_some()
    }
}