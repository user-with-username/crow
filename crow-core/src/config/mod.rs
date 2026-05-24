use anyhowed::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod archiver;
mod build;
mod compiler;
mod dependencies;
mod formatter;
mod linker;
mod macros;
mod package;
pub mod profile;
pub mod r#type;
mod workspace;
pub use archiver::ArchiverConfig;

pub use build::BuildConfig;
pub use compiler::CompilerConfig;
pub use dependencies::{Dependencies, DependencySource, DependencySpec};
pub use formatter::FormatterConfig;
pub use linker::LinkerConfig;
pub use package::Package;
pub use profile::{BenchProfile, DevProfile, Profile, Profiles, ReleaseProfile, TestProfile};
pub use r#type::{BinaryConfig, LibraryConfig, ProjectType, TargetType};
pub use workspace::Workspace;

#[derive(Deserialize, Debug, Clone)]
pub struct CrowConfig {
    /// Package definition. Optional for virtual manifests (workspaces)
    pub package: Option<Package>,

    #[serde(default)]
    pub workspace: Option<workspace::WorkspaceConfig>,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub dependencies: Dependencies,

    #[serde(default)]
    pub profile: Profiles,
}

impl CrowConfig {
    /// Loads configuration from a specific file path.
    pub fn load_from(dir: &Path, is_dep: bool) -> Result<(Self, PathBuf)> {
        let config_path = dir.join("crow.toml");

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config at {}", config_path.display()))?;

        let mut config: Self = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        if !has_explicit_package_type(&content)? && is_dep {
            if let Some(package) = config.package.as_mut() {
                package.r#type = ProjectType::StaticLib(Default::default());
            }
        }

        Ok((config, dir.to_path_buf()))
    }

    /// Recursively searches upwards for `crow.toml`.
    pub fn find_in_tree(start_path: &Path) -> Result<(Self, PathBuf)> {
        let mut check_path = start_path.to_path_buf();

        loop {
            let config_path = check_path.join("crow.toml");

            if config_path.exists() {
                return Self::load_from(&check_path, false);
            }

            if let Some(parent) = check_path.parent() {
                check_path = parent.to_path_buf();
            } else {
                break;
            }
        }

        anyhowed::bail!(
            "could not find `crow.toml` in {} or any parent directory",
            start_path.display()
        );
    }

    /// Checks if this is a virtual manifest (workspace without a root package).
    pub fn is_virtual(&self) -> bool {
        self.package.is_none() && self.workspace.is_some()
    }
}

fn has_explicit_package_type(content: &str) -> Result<bool> {
    let value: toml::Value = toml::from_str(content).context("failed to parse TOML value")?;
    Ok(value
        .get("package")
        .and_then(|pkg| pkg.get("type"))
        .is_some())
}

pub fn parse_standard(standard: Option<&str>) -> Option<u32> {
    standard.and_then(|s| s.trim().parse::<u32>().ok())
}
