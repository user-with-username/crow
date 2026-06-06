pub use semver::{Version, VersionReq};

use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct Dependencies(pub BTreeMap<String, DependencySpec>);

impl Deref for Dependencies {
    type Target = BTreeMap<String, DependencySpec>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// foo = "1.2.3"
    Version(VersionReq),

    /// foo = { git = "...", build_flags = [...] }
    Git {
        git: String,

        #[serde(default)]
        build_flags: Vec<String>,
    },

    /// foo = { path = "../foo", build_flags = [...] }
    Path {
        path: PathBuf,

        #[serde(default)]
        build_flags: Vec<String>,
    },

    /// foo = { version = "^1.0", registry = "my-registry" }
    Registry {
        version: VersionReq,

        #[serde(default)]
        registry: Option<String>,

        #[serde(default)]
        build_flags: Vec<String>,
    },

    /// foo = { system = true, libs = ["ssl", "crypto"] }
    System {
        system: bool,

        #[serde(default)]
        libs: Vec<String>,
    },
}

impl DependencySpec {
    pub fn matches_version(&self, version: &Version) -> bool {
        match self {
            Self::Version(req) => req.matches(version),

            Self::Registry { version: req, .. } => {
                req.matches(version)
            }

            _ => false,
        }
    }

    pub fn version_req(&self) -> Option<&VersionReq> {
        match self {
            Self::Version(req) => Some(req),

            Self::Registry { version, .. } => Some(version),

            _ => None,
        }
    }

    pub fn build_flags(&self) -> &[String] {
        match self {
            Self::Git { build_flags, .. }
            | Self::Path { build_flags, .. }
            | Self::Registry { build_flags, .. } => build_flags,

            _ => &[],
        }
    }

    pub fn is_system(&self) -> bool {
        matches!(self, Self::System { .. })
    }

    pub fn system_libs(&self) -> &[String] {
        match self {
            Self::System { libs, .. } => libs,
            _ => &[],
        }
    }
}