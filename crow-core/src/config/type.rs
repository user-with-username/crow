use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, SmartDefault)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ProjectType {
    #[default]
    Bin(BinaryConfig),
    Exe(BinaryConfig),
    Lib(LibraryConfig),
    StaticLib(LibraryConfig),
    SharedLib(LibraryConfig),
    Module(LibraryConfig),
    HeaderOnly,
}

impl ProjectType {
    pub fn is_bin(&self) -> bool {
        matches!(self, ProjectType::Bin(_) | ProjectType::Exe(_))
    }

    pub fn is_lib(&self) -> bool {
        matches!(
            self,
            ProjectType::Lib(_)
                | ProjectType::StaticLib(_)
                | ProjectType::SharedLib(_)
                | ProjectType::Module(_)
                | ProjectType::HeaderOnly
        )
    }

    pub fn is_static(&self) -> bool {
        matches!(self, ProjectType::Lib(_) | ProjectType::StaticLib(_))
    }

    pub fn is_shared(&self) -> bool {
        matches!(self, ProjectType::SharedLib(_) | ProjectType::Module(_))
    }

    pub fn display_name<'a>(&'a self, package_name: &'a str) -> &'a str {
        match self {
            ProjectType::Bin(cfg) | ProjectType::Exe(cfg) => {
                cfg.name.as_deref().unwrap_or(package_name)
            }
            ProjectType::Lib(cfg)
            | ProjectType::StaticLib(cfg)
            | ProjectType::SharedLib(cfg)
            | ProjectType::Module(cfg) => cfg.name.as_deref().unwrap_or(package_name),
            ProjectType::HeaderOnly => package_name,
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            ProjectType::Bin(_) => "bin",
            ProjectType::Exe(_) => "exe",
            ProjectType::Lib(_) => "lib",
            ProjectType::StaticLib(_) => "static-lib",
            ProjectType::SharedLib(_) => "shared-lib",
            ProjectType::Module(_) => "module",
            ProjectType::HeaderOnly => "header-only",
        }
    }

    pub fn output_name(&self, package_name: &str) -> String {
        match self {
            ProjectType::Bin(cfg) => cfg.name.clone().unwrap_or_else(|| package_name.to_string()),
            ProjectType::Exe(cfg) => cfg.name.clone().unwrap_or_else(|| package_name.to_string()),
            ProjectType::Lib(cfg) => Self::lib_name(package_name, cfg, "lib"),
            ProjectType::StaticLib(cfg) => Self::lib_name(package_name, cfg, "lib"),
            ProjectType::SharedLib(cfg) => Self::lib_name(package_name, cfg, "lib"),
            ProjectType::Module(cfg) => Self::lib_name(package_name, cfg, "module"),
            ProjectType::HeaderOnly => package_name.to_string(),
        }
    }

    fn lib_name(package_name: &str, cfg: &LibraryConfig, prefix: &str) -> String {
        cfg.name.clone().unwrap_or_else(|| {
            if cfg.prefix.unwrap_or(true) {
                format!("{}{}", prefix, package_name)
            } else {
                package_name.to_string()
            }
        })
    }

    pub fn entry_points(&self) -> Vec<String> {
        match self {
            ProjectType::Bin(cfg) => cfg
                .entry
                .clone()
                .unwrap_or_else(|| vec!["main.cpp".to_string()]),
            ProjectType::Exe(cfg) => cfg
                .entry
                .clone()
                .unwrap_or_else(|| vec!["main.cpp".to_string()]),
            ProjectType::Lib(cfg) => cfg
                .sources
                .clone()
                .unwrap_or_else(|| vec!["lib.cpp".to_string()]),
            ProjectType::StaticLib(cfg) => cfg
                .sources
                .clone()
                .unwrap_or_else(|| vec!["lib.cpp".to_string()]),
            ProjectType::SharedLib(cfg) => cfg
                .sources
                .clone()
                .unwrap_or_else(|| vec!["lib.cpp".to_string()]),
            ProjectType::Module(cfg) => cfg
                .sources
                .clone()
                .unwrap_or_else(|| vec!["module.cpp".to_string()]),
            ProjectType::HeaderOnly => vec![],
        }
    }

    pub fn version_script(&self) -> Option<PathBuf> {
        match self {
            ProjectType::SharedLib(cfg) => cfg.version_script.clone(),
            ProjectType::Module(cfg) => cfg.version_script.clone(),
            _ => None,
        }
    }

    pub fn generate_import_lib(&self) -> bool {
        match self {
            ProjectType::SharedLib(cfg) => cfg.generate_import_lib.unwrap_or(true),
            ProjectType::Module(cfg) => cfg.generate_import_lib.unwrap_or(false),
            _ => false,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ProjectType::Bin(_) | ProjectType::Exe(_) => {
                if cfg!(target_os = "windows") {
                    "exe"
                } else {
                    ""
                }
            }
            ProjectType::SharedLib(_) | ProjectType::Module(_) => {
                if cfg!(target_os = "windows") {
                    "dll"
                } else {
                    "so"
                }
            }
            ProjectType::Lib(_) | ProjectType::StaticLib(_) => {
                if cfg!(target_os = "windows") {
                    "lib"
                } else {
                    "a"
                }
            }
            ProjectType::HeaderOnly => "",
        }
    }

    pub fn needs_prefix(&self) -> bool {
        match self {
            ProjectType::Lib(cfg) | ProjectType::StaticLib(cfg) | ProjectType::SharedLib(cfg) => {
                cfg.prefix.unwrap_or(true)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
pub struct BinaryConfig {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub entry: Option<Vec<String>>,

    #[serde(default)]
    pub main: Option<String>,

    #[serde(default)]
    pub console: Option<bool>,

    #[serde(default)]
    pub subsystem: Option<String>,

    #[serde(default)]
    pub icon: Option<PathBuf>,

    #[serde(default)]
    pub manifest: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
pub struct LibraryConfig {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub sources: Option<Vec<String>>,

    #[serde(default)]
    pub prefix: Option<bool>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub soversion: Option<String>,

    #[serde(default)]
    pub version_script: Option<PathBuf>,

    #[serde(default)]
    pub generate_import_lib: Option<bool>,

    #[serde(default)]
    pub whole_archive: Option<bool>,

    #[serde(default)]
    pub no_builtin_libs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    Executable,
    StaticLibrary,
    SharedLibrary,
    Module,
    HeaderOnly,
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::Executable
    }
}

crate::config::macros::type_enum!();
