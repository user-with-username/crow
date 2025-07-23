use crate::dependency::crow_dependency_build::CrowDependencyBuild;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Dependency {
    Git {
        git: String,
        #[serde(default = "Dependency::default_branch")]
        branch: String,
        #[serde(default)]
        build: Option<CrowDependencyBuild>,
    },
    Path {
        path: String,
        #[serde(default)]
        build: Option<CrowDependencyBuild>,
    },
}

impl Dependency {
    fn default_branch() -> String {
        "".to_string()
    }
}
