use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct Dependencies(pub BTreeMap<String, DependencySpec>);

impl Dependencies {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &DependencySpec)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    ShorthandGit(String),
    Detailed(DependencySource),
}

impl DependencySpec {
    pub fn source(&self) -> DependencySource {
        match self {
            Self::ShorthandGit(url) => DependencySource {
                git: Some(url.clone()),
                path: None,
                registry: None,
            },
            Self::Detailed(source) => source.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DependencySource {
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub registry: Option<String>,
}

impl DependencySource {
    pub fn kind(&self) -> &'static str {
        if self.git.is_some() {
            "git"
        } else if self.path.is_some() {
            "path"
        } else if self.registry.is_some() {
            "registry"
        } else {
            "unknown"
        }
    }
}
