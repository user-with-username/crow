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
use crow_utils::condition::{
    apply_target_filter, get_named_targets_list, has_named_targets, TargetInfo,
};
pub use dependencies::{Dependencies, DependencySpec, DevDependencies};
pub use formatter::FormatterConfig;
pub use linker::LinkerConfig;
pub use package::Package;
pub use profile::{BenchProfile, DevProfile, Profile, Profiles, ReleaseProfile, TestProfile};
pub use r#type::{BinaryConfig, LibraryConfig, ProjectType, TargetType};
pub use workspace::Workspace;

#[derive(Deserialize, Debug, Clone)]
pub struct CrowConfig {
    /// Package definition. Optional for virtual manifests (workspaces).
    pub package: Option<Package>,

    #[serde(default)]
    pub workspace: Option<workspace::WorkspaceConfig>,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub dependencies: Dependencies,

    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: DevDependencies,

    #[serde(default)]
    pub profile: Profiles,
}

impl CrowConfig {
    /// Loads configuration from a directory containing `crow.toml`.
    pub fn load_from(
        dir: &Path,
        is_dep: bool,
        target_name: Option<&str>,
    ) -> Result<(Self, PathBuf)> {
        let config_path = dir.join("crow.toml");

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config at {}", config_path.display()))?;

        if !is_dep && target_name.is_none() && has_named_targets(&content) {
            let targets = get_named_targets_list(&content);

            if targets.len() == 1 {
                anyhowed::bail!(
                    "named target `{}` is defined in [target] section\n\
                     Please specify it using `--target {}`",
                    targets[0],
                    targets[0]
                );
            } else {
                let target_list = targets.join(", ");
                anyhowed::bail!(
                    "multiple named targets defined in [target] section: {}\n\
                     Please specify which target to build using `--target <name>`",
                    target_list
                );
            }
        }

        let processed = if let Some(target) = target_name {
            apply_target_filter(&content, target)
                .map_err(|e| anyhowed::Error::msg(e))
                .with_context(|| {
                    format!(
                        "failed to apply target '{}' in {}",
                        target,
                        config_path.display()
                    )
                })?
        } else {
            let target_info = TargetInfo::current();
            apply_target_filter(&content, &format!("\"{}\"", target_info.triple))
                .map_err(|e| anyhowed::Error::msg(e))
                .with_context(|| {
                    format!(
                        "failed to process target-specific config in {}",
                        config_path.display()
                    )
                })?
        };

        let mut config: Self = toml::from_str(&processed)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        if !has_explicit_package_type(&processed)? && is_dep {
            if let Some(package) = config.package.as_mut() {
                package.r#type = ProjectType::StaticLib(Default::default());
            }
        }

        Ok((config, dir.to_path_buf()))
    }

    /// Recursively searches upwards for `crow.toml`.
    pub fn find_in_tree(start_path: &Path, target_name: Option<&str>) -> Result<(Self, PathBuf)> {
        let mut check_path = start_path.to_path_buf();

        loop {
            let config_path = check_path.join("crow.toml");

            if config_path.exists() {
                return Self::load_from(&check_path, false, target_name);
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