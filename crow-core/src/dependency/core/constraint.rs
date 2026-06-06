use crate::config::DependencySpec;
use anyhowed::Result;
pub use semver::{Version, VersionReq};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Git,
    Path,
    Registry,
    System,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Path => "path",
            Self::Registry => "registry",
            Self::System => "system",
        }
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyConstraint {
    pub source: Option<String>,
    pub version: Option<VersionReq>,
    pub source_type: SourceType,
}

impl DependencyConstraint {
    pub fn new(
        source: Option<String>,
        version: Option<VersionReq>,
        source_type: SourceType,
    ) -> Self {
        let version = if source_type == SourceType::System { None } else { version };

        Self {
            source,
            version,
            source_type,
        }
    }

    pub fn from_dependency_spec(spec: &DependencySpec, owner_root: &Path) -> Result<Self> {
        match spec {
            DependencySpec::Git { git, .. } => {
                Ok(Self::new(Some(git.clone()), None, SourceType::Git))
            }
            DependencySpec::Version(version) => {
                Ok(Self::new(None, Some(version.clone()), SourceType::Registry))
            }
            DependencySpec::Registry { version, registry, .. } => {
                let registry_url = registry
                    .clone()
                    .unwrap_or_else(|| crow_utils::environment::DEFAULT_REGISTRY_URL.to_string());
                
                Ok(Self::new(
                    Some(registry_url),
                    Some(version.clone()),
                    SourceType::Registry,
                ))
            }
            DependencySpec::Path { path, .. } => {
                let abs_path = if path.is_relative() {
                    owner_root.join(path)
                } else {
                    path.clone()
                };
                
                let canonical = abs_path.canonicalize().unwrap_or(abs_path);
                
                Ok(Self::new(
                    Some(canonical.to_string_lossy().into_owned()),
                    None,
                    SourceType::Path,
                ))
            }
            DependencySpec::System { .. } => {
                Ok(Self::new(None, None, SourceType::System))
            }
        }
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        self != other
    }

    pub fn display_short(&self) -> String {
        let fallback = "?";
        match self.source_type {
            SourceType::Git | SourceType::Path => {
                format!("{}: {}", self.source_type, self.source.as_deref().unwrap_or(fallback))
            }
            SourceType::Registry => {
                let version_str = self.version
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| fallback.to_string());
                format!("registry: {version_str}")
            }
            SourceType::System => "system".to_string(),
        }
    }
}