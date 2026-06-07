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
mod target;
pub mod r#type;
mod workspace;

pub use archiver::ArchiverConfig;
pub use build::BuildConfig;
pub use compiler::CompilerConfig;
use crow_utils::condition::{
    apply_build_target_sections, apply_target_filter, extract_build_targets, TargetInfo,
};
pub use dependencies::{Dependencies, DependencySpec};
pub use formatter::FormatterConfig;
pub use linker::LinkerConfig;
pub use package::Package;
pub use profile::{BenchProfile, DevProfile, Profile, Profiles, ReleaseProfile, TestProfile};
pub use r#type::{BinaryConfig, LibraryConfig, ProjectType, TargetType};
pub use target::BuildTargets;
pub use workspace::Workspace;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CrowConfig {
    /// Package definition. Optional for virtual manifests (workspaces).
    pub package: Option<Package>,

    #[serde(default)]
    pub workspace: Option<workspace::WorkspaceConfig>,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub dependencies: Dependencies,

    #[serde(default)]
    pub profile: Profiles,

    #[serde(skip, default)]
    pub build_targets: BuildTargets,

    #[serde(skip, default)]
    pub(crate) processed_toml: String,
}

impl CrowConfig {
    /// Returns a copy of this config with the named build target's section overrides applied.
    pub fn with_build_target(&self, name: &str) -> Result<Self> {
        let sections = self.build_targets.get(name).ok_or_else(|| {
            let available: Vec<&str> = self.build_targets.keys().map(String::as_str).collect();
            if available.is_empty() {
                anyhowed::anyhow!(
                    "Unknown build target `{}`. No build targets defined in crow.toml",
                    name
                )
            } else {
                anyhowed::anyhow!(
                    "Unknown build target `{}`. Available targets: {}",
                    name,
                    available.join(", ")
                )
            }
        })?;

        let merged = apply_build_target_sections(&self.processed_toml, sections)
            .map_err(|e| anyhowed::Error::msg(e))
            .with_context(|| format!("failed to apply build target `{}`", name))?;

        let mut config: Self = toml::from_str(&merged)
            .with_context(|| format!("failed to parse config for target `{}`", name))?;
        config.build_targets = self.build_targets.clone();
        config.processed_toml = merged;
        Ok(config)
    }

    /// Loads configuration from a directory containing `crow.toml`.
    pub fn load_from(dir: &Path, is_dep: bool) -> Result<(Self, PathBuf)> {
        let config_path = dir.join("crow.toml");

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config at {}", config_path.display()))?;

        let (build_targets, content) = extract_build_targets(&content)
            .map_err(|e| anyhowed::Error::msg(e))
            .with_context(|| {
                format!(
                    "failed to extract build targets from {}",
                    config_path.display()
                )
            })?;

        let target_info = TargetInfo::current();
        let processed = apply_target_filter(&content, &target_info)
            .map_err(|e| anyhowed::Error::msg(e))
            .with_context(|| {
                format!(
                    "failed to process target-specific config in {}",
                    config_path.display()
                )
            })?;

        let mut config: Self = toml::from_str(&processed)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        let is_dep_without_type = !has_explicit_package_type(&processed)? && is_dep;

        config.build_targets = build_targets;
        config.processed_toml = processed;

        if is_dep_without_type {
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

            match check_path.parent() {
                Some(parent) => check_path = parent.to_path_buf(),
                None => break,
            }
        }

        anyhowed::bail!(
            "could not find `crow.toml` in {} or any parent directory",
            start_path.display()
        );
    }

    /// Returns `true` if this is a virtual manifest (workspace root without a package).
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
