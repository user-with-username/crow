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
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    
    pub fn get(&self, key: &str) -> Option<&DependencySpec> {
        self.0.get(key)
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    ShorthandGit(String),
    Detailed(DependencySource),
    System(SystemDependency),
}

impl DependencySpec {
    pub fn source(&self) -> Option<DependencySource> {
        match self {
            Self::ShorthandGit(url) => Some(DependencySource {
                git: Some(url.clone()),
                path: None,
                registry: None,
                build_flags: Vec::new(),
            }),
            Self::Detailed(source) => Some(source.clone()),
            Self::System(_) => None,
        }
    }

    pub fn is_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

    pub fn system_libs(&self) -> Vec<String> {
        match self {
            Self::System(sys) => sys.libs.clone(),
            _ => Vec::new(),
        }
    }
    
    pub fn is_shorthand_git(&self) -> bool {
        matches!(self, Self::ShorthandGit(_))
    }
    
    pub fn is_detailed(&self) -> bool {
        matches!(self, Self::Detailed(_))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemDependency {
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub libs: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencySource {
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub build_flags: Vec<String>,
}

impl DependencySource {
    pub fn has_git(&self) -> bool {
        self.git.is_some()
    }
    
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
    
    pub fn has_registry(&self) -> bool {
        self.registry.is_some()
    }
    
    pub fn source_type(&self) -> &'static str {
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