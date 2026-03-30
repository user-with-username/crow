use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}

impl Package {
    /// Creates a new package with the given name and version.
    /// Useful for testing or programmatic creation.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            authors: None,
            description: None,
            license: None,
            repository: None,
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