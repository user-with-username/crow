use crate::config::{DependencySource, DependencySpec};
use anyhowed::{bail, Result};
pub use semver::{Version, VersionReq};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct DependencyConstraint {
    pub source: Option<String>,
    pub version: Option<VersionReq>,
    pub source_type: String,
}

impl DependencyConstraint {
    pub fn new(
        source: Option<String>,
        version: Option<String>,
        source_type: String,
    ) -> Result<Self> {
        let parsed_version = match version {
            Some(v) if source_type != "system" => Some(VersionReq::parse(&v)?),
            None => None,
            Some(_) => None,
        };
        Ok(Self {
            source,
            version: parsed_version,
            source_type,
        })
    }

    pub fn from_dependency_spec(
        dep_name: &str,
        spec: &DependencySpec,
        owner_root: &Path,
    ) -> Result<Self> {
        match spec {
            DependencySpec::ShorthandGit(url) => {
                Self::new(Some(url.clone()), None, "git".to_string())
            }
            DependencySpec::ShorthandVersion(version) => {
                Self::new(None, Some(version.clone()), "registry".to_string())
            }
            DependencySpec::Detailed(source) => {
                Self::from_dependency_source(dep_name, source, owner_root)
            }
            DependencySpec::System(_) => Self::new(None, None, "system".to_string()),
        }
    }

    fn from_dependency_source(
        dep_name: &str,
        source: &DependencySource,
        owner_root: &Path,
    ) -> Result<Self> {
        if let Some(git_url) = &source.git {
            Self::new(
                Some(git_url.clone()),
                source.version.clone(),
                "git".to_string(),
            )
        } else if let Some(path) = &source.path {
            let abs_path = if path.is_relative() {
                owner_root.join(path)
            } else {
                path.clone()
            };
            let canonical = abs_path.canonicalize().unwrap_or(abs_path);
            Self::new(
                Some(canonical.display().to_string()),
                None,
                "path".to_string(),
            )
        } else if source.version.is_some() || source.registry.is_some() {
            let registry_url = source
                .registry
                .clone()
                .unwrap_or_else(|| crow_utils::environment::DEFAULT_REGISTRY_URL.to_string());
            Self::new(
                Some(registry_url),
                source.version.clone(),
                "registry".to_string(),
            )
        } else {
            bail!("dependency `{dep_name}` has no valid source specification")
        }
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        if self.source_type != other.source_type {
            return true;
        }
        match (&self.source, &other.source) {
            (Some(s1), Some(s2)) if s1 != s2 => return true,
            _ => {}
        }

        match (&self.version, &other.version) {
            (Some(v1), Some(v2)) => v1.to_string() != v2.to_string(),
            _ => false,
        }
    }

    pub fn display_short(&self) -> String {
        match self.source_type.as_str() {
            "git" => format!("git: {}", self.source.as_deref().unwrap_or("?")),
            "path" => format!("path: {}", self.source.as_deref().unwrap_or("?")),
            "registry" => format!(
                "registry: {}",
                self.version
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "?".to_string())
            ),
            "system" => "system".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
