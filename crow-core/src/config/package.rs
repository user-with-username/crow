use serde::Deserialize;
use smart_default::SmartDefault;

use crate::config::ProjectType;

#[derive(Deserialize, Debug, Clone, SmartDefault)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,

    #[serde(default)]
    #[default(ProjectType::default())]
    pub r#type: ProjectType,
}

impl Package {
    /// Creates a new package with the given name and version.
    /// Useful for testing or programmatic creation.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            ..Default::default()
        }
    }

    /// Returns the package name as a string slice.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the package version as a string slice.
    pub fn version(&self) -> &str {
        &self.version
    }
}
