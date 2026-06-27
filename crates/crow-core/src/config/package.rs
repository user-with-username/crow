use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::{Path, PathBuf};

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

    #[serde(default)]
    pub standard: Option<String>,
}

impl Package {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn standard(&self) -> Option<&str> {
        self.standard.as_deref()
    }

    pub fn output_stem(&self) -> String {
        let base_name = self.r#type.display_name(&self.name);
        if self.r#type.needs_prefix() {
            format!("lib{}", base_name)
        } else {
            base_name.to_string()
        }
    }

    pub fn output_name(&self) -> String {
        let stem = self.output_stem();
        let extension = self.r#type.extension();
        if extension.is_empty() {
            stem
        } else {
            format!("{}.{}", stem, extension)
        }
    }

    pub fn output_path_in(&self, dir: &Path) -> PathBuf {
        dir.join(self.output_stem())
            .with_extension(self.r#type.extension())
    }
}
