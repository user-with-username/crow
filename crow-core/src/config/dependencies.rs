use serde::Deserialize;
use serde::Deserializer;
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

#[derive(Debug, Clone)]
pub enum DependencySpec {
    /// Shorthand: just a git URL string, `dep = "https://github.com/foo/bar"`
    ShorthandGit(String),
    /// Shorthand: version string, `dep = "1.0.0"`
    ShorthandVersion(String),
    /// Detailed table, `[dependencies.dep]  git = "..."` or `version = "1.0.0"`
    Detailed(DependencySource),
    /// System dependency, `[dependencies.openssl]  system = true`
    System(SystemDependency),
}

impl<'de> Deserialize<'de> for DependencySpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        
        let value = serde_json::Value::deserialize(deserializer)?;
        
        if let Some(s) = value.as_str() {
            // git url?
            if is_git_url(s) {
                return Ok(DependencySpec::ShorthandGit(s.to_string()));
            }
            if looks_like_version(s) {
                return Ok(DependencySpec::ShorthandVersion(s.to_string()));
            }
            Ok(DependencySpec::ShorthandGit(s.to_string()))
        } else if value.is_object() {
            let source: DependencySource = serde::Deserialize::deserialize(value)
                .map_err(Error::custom)?;
            
            if let Some(true) = source.system {
                Ok(DependencySpec::System(SystemDependency {
                    system: true,
                    libs: source.libs.clone(),
                }))
            } else {
                Ok(DependencySpec::Detailed(source))
            }
        } else {
            Err(Error::custom("expected string or table for dependency"))
        }
    }
}

fn is_git_url(s: &str) -> bool {
    s.starts_with("http://") 
        || s.starts_with("https://")
        || s.starts_with("git@")
        || s.starts_with("git://")
        || s.contains(".git")
        || (s.contains('/') && !looks_like_version(s))
}

fn looks_like_version(s: &str) -> bool {
    let first = s.chars().next().unwrap_or('\0');
    if first.is_ascii_digit() {
        return true;
    }
    matches!(first, '^' | '~' | '>' | '<' | '=')
}

impl DependencySpec {
    pub fn source(&self) -> Option<DependencySource> {
        match self {
            Self::ShorthandGit(url) => Some(DependencySource {
                git: Some(url.clone()),
                path: None,
                registry: None,
                version: None,
                build_flags: Vec::new(),
                system: None,
                libs: Vec::new(),
            }),
            Self::ShorthandVersion(version) => Some(DependencySource {
                git: None,
                path: None,
                registry: None,
                version: Some(version.clone()),
                build_flags: Vec::new(),
                system: None,
                libs: Vec::new(),
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
    
    pub fn is_shorthand_version(&self) -> bool {
        matches!(self, Self::ShorthandVersion(_))
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
#[serde(deny_unknown_fields, default)]
pub struct DependencySource {
    pub git: Option<String>,

    pub path: Option<PathBuf>,

    pub registry: Option<String>,

    pub version: Option<String>,

    #[serde(default)]
    pub build_flags: Vec<String>,
    
    #[serde(default)]
    pub system: Option<bool>,

    #[serde(default)]
    pub libs: Vec<String>,
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

    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    pub fn source_type(&self) -> &'static str {
        if self.git.is_some() {
            "git"
        } else if self.path.is_some() {
            "path"
        } else if self.version.is_some() {
            "registry"
        } else if self.registry.is_some() {
            "registry"
        } else {
            "unknown"
        }
    }
}