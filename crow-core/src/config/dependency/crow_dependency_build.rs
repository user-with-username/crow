use crate::build_system_type::BuildSystemType;
use crate::output_type::OutputType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct CrowDependencyBuild {
    pub output_type: OutputType,
    pub build_system: Option<BuildSystemType>,
    pub cmake_options: Vec<String>,
    pub lib_name: String,
    pub pch_headers: Vec<String>,
}

impl Default for CrowDependencyBuild {
    fn default() -> Self {
        CrowDependencyBuild {
            output_type: Self::default_output_type(),
            build_system: None,
            cmake_options: Vec::new(),
            lib_name: Self::default_lib_name_placeholder(),
            pch_headers: Vec::new(),
        }
    }
}

impl CrowDependencyBuild {
    fn default_output_type() -> OutputType {
        OutputType::StaticLib
    }

    fn default_lib_name_placeholder() -> String {
        String::from("__INFER_LIB_NAME__")
    }

    pub fn infer_defaults(
        dep_path: &std::path::Path,
        dep_name: &str,
        existing_config: Option<Self>,
    ) -> Self {
        let mut config = existing_config.unwrap_or_default();

        if config.build_system.is_none() {
            if dep_path.join("crow.toml").exists() {
                config.build_system = Some(BuildSystemType::Crow);
            } else {
                config.build_system = Some(BuildSystemType::Cmake);
            }
        }

        if config.lib_name == CrowDependencyBuild::default_lib_name_placeholder() {
            config.lib_name = dep_name.to_string();
        }
        config
    }
}
