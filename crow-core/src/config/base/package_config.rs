use crate::output_type::OutputType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub output_type: OutputType,
    pub sources: Vec<String>,
    pub includes: Vec<String>,
    pub libs: Vec<String>,
    pub lib_dirs: Vec<String>,
}

impl PackageConfig {
    fn default_sources() -> Vec<String> {
        vec!["src/**/*.cpp".to_string(), "src/**/*.c".to_string()]
    }

    fn default_includes() -> Vec<String> {
        vec!["include/".to_string()]
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        PackageConfig {
            name: String::new(),
            version: String::new(),
            output_type: OutputType::default(),
            sources: Self::default_sources(),
            includes: Self::default_includes(),
            libs: Vec::new(),
            lib_dirs: Vec::new(),
        }
    }
}
