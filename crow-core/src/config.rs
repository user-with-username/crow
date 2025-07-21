use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum OutputType {
    #[serde(rename = "executable")]
    Executable,
    #[serde(rename = "static-lib")]
    StaticLib,
    #[serde(rename = "shared-lib")]
    SharedLib,
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::Executable
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum BuildSystemType {
    #[serde(rename = "crow")]
    Crow,
    #[serde(rename = "cmake")]
    Cmake,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub package: PackageConfig,
    pub toolchain: ToolchainConfig,
    pub profiles: Option<HashMap<String, BuildProfile>>,
    pub targets: HashMap<String, Target>,
    pub dependencies: HashMap<String, Dependency>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            package: PackageConfig::default(),
            toolchain: ToolchainConfig::default(),
            profiles: Some(Config::generate_default_profiles_map()),
            targets: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| "Cannot load `crow.toml`. Is it crow project?")?;
        let mut config: Config = toml::from_str(&content)?;

        if config.profiles.is_none() {
            config.profiles = Some(Config::generate_default_profiles_map());
        } else {
            let mut profiles = config.profiles.unwrap_or_default();

            let default_profiles = Config::generate_default_profiles_map();
            for name in ["debug", "release"] {
                if !profiles.contains_key(name) {
                    profiles.insert(
                        name.to_string(),
                        default_profiles.get(name).unwrap().clone(),
                    );
                }
            }

            config.profiles = Some(profiles);
        }
        Ok(config)
    }

    pub fn final_config(&self, target_name: &str, profile_name: &str) -> FinalConfig {
        let package = &self.package;
        let target = self.targets.get(target_name);
        let profile = self
            .profiles
            .as_ref()
            .and_then(|p| p.get(profile_name))
            .cloned()
            .unwrap_or_default();

        FinalConfig {
            name: target
                .and_then(|t| t.name.clone())
                .unwrap_or_else(|| package.name.clone()),
            version: package.version.clone(),
            output_type: target
                .and_then(|t| t.output_type.clone())
                .unwrap_or(package.output_type.clone()),
            sources: target
                .and_then(|t| t.sources.clone())
                .unwrap_or_else(|| package.sources.clone()),
            includes: target
                .and_then(|t| t.includes.clone())
                .unwrap_or_else(|| package.includes.clone()),
            libs: target
                .and_then(|t| t.libs.clone())
                .unwrap_or_else(|| package.libs.clone()),
            lib_dirs: target
                .and_then(|t| t.lib_dirs.clone())
                .unwrap_or_else(|| package.lib_dirs.clone()),

            opt_level: target
                .and_then(|t| t.opt_level)
                .unwrap_or(profile.opt_level),
            defines: target
                .and_then(|t| t.defines.clone())
                .unwrap_or(profile.defines),
            lto: target.and_then(|t| t.lto).unwrap_or(profile.lto),
            flags: target
                .and_then(|t| t.flags.clone())
                .unwrap_or(profile.flags),
            incremental: target
                .and_then(|t| t.incremental)
                .unwrap_or(profile.incremental),

            toolchain: self
                .toolchain
                .merge(target.and_then(|t| t.toolchain.as_ref())),
        }
    }

    fn generate_default_profiles_map() -> HashMap<String, BuildProfile> {
        let mut profiles = HashMap::new();
        profiles.insert("debug".to_string(), BuildProfile::default_debug());
        profiles.insert("release".to_string(), BuildProfile::default_release());
        profiles
    }
}

#[derive(Debug, Clone)]
pub struct FinalConfig {
    pub name: String,
    pub version: String,
    pub output_type: OutputType,
    pub sources: Vec<String>,
    pub includes: Vec<String>,
    pub libs: Vec<String>,
    pub lib_dirs: Vec<String>,

    pub opt_level: u8,
    pub defines: Vec<String>,
    pub lto: bool,
    pub flags: Vec<String>,
    pub incremental: bool,

    pub toolchain: ToolchainConfig,
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ToolchainConfig {
    pub compiler: String,
    pub compiler_flags: Vec<String>,
    pub linker: String,
    pub linker_flags: Vec<String>,
    pub archiver: String,
    pub archiver_flags: Vec<String>,
    pub hooks: Option<Vec<String>>,
}

impl ToolchainConfig {
    fn default_archiver() -> String {
        if cfg!(windows) {
            "lib.exe".to_string()
        } else {
            "ar".to_string()
        }
    }

    fn default_archiver_flags() -> Vec<String> {
        if cfg!(windows) {
            vec![]
        } else {
            vec!["rcs".to_string()]
        }
    }

    fn default_compiler_flags() -> Vec<String> {
        vec!["-std=c++17".to_string()]
    }

    fn default_linker_flags() -> Vec<String> {
        vec!["-lstdc++".to_string()]
    }

    fn find_default_compiler_and_linker() -> anyhow::Result<(String, String)> {
        if std::process::Command::new("clang++")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(("clang++".to_string(), "clang++".to_string()));
        }
        if std::process::Command::new("g++")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(("g++".to_string(), "g++".to_string()));
        }
        anyhow::bail!("Cannot find compiler in PATH.");
    }

    fn merge(&self, override_config: Option<&ToolchainOverride>) -> Self {
        let Some(ov) = override_config else {
            return self.clone();
        };

        ToolchainConfig {
            compiler: ov.compiler.clone().unwrap_or(self.compiler.clone()),
            compiler_flags: ov
                .compiler_flags
                .clone()
                .unwrap_or(self.compiler_flags.clone()),
            linker: ov.linker.clone().unwrap_or(self.linker.clone()),
            linker_flags: ov.linker_flags.clone().unwrap_or(self.linker_flags.clone()),
            archiver: ov.archiver.clone().unwrap_or(self.archiver.clone()),
            archiver_flags: ov
                .archiver_flags
                .clone()
                .unwrap_or(self.archiver_flags.clone()),
            hooks: ov.hooks.clone().or(self.hooks.clone()),
        }
    }
}

impl Default for ToolchainConfig {
    fn default() -> Self {
        let (compiler, linker) = ToolchainConfig::find_default_compiler_and_linker()
            .unwrap_or(("g++".to_string(), "g++".to_string()));
        ToolchainConfig {
            compiler,
            linker,
            compiler_flags: ToolchainConfig::default_compiler_flags(),
            linker_flags: ToolchainConfig::default_linker_flags(),
            archiver: ToolchainConfig::default_archiver(),
            archiver_flags: ToolchainConfig::default_archiver_flags(),
            hooks: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BuildProfile {
    pub opt_level: u8,
    pub defines: Vec<String>,
    pub lto: bool,
    pub flags: Vec<String>,
    pub incremental: bool,
}

impl Default for BuildProfile {
    fn default() -> Self {
        BuildProfile::default_debug()
    }
}

impl BuildProfile {
    fn default_debug() -> Self {
        BuildProfile {
            opt_level: 0,
            defines: vec!["DEBUG".to_string()],
            lto: false,
            flags: vec!["-g".to_string()],
            incremental: true,
        }
    }

    fn default_release() -> Self {
        BuildProfile {
            opt_level: 3,
            defines: vec!["NDEBUG".to_string()],
            lto: true,
            flags: vec!["-O3".to_string()],
            incremental: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ToolchainOverride {
    pub compiler: Option<String>,
    pub compiler_flags: Option<Vec<String>>,
    pub linker: Option<String>,
    pub linker_flags: Option<Vec<String>>,
    pub archiver: Option<String>,
    pub archiver_flags: Option<Vec<String>>,
    pub hooks: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Target {
    pub os: Option<String>,
    pub arch: Option<String>,
    pub os_version: Option<String>,
    pub hooks: Option<Vec<String>>,
    pub toolchain: Option<ToolchainOverride>,
    pub name: Option<String>,
    pub output_type: Option<OutputType>,
    pub sources: Option<Vec<String>>,
    pub includes: Option<Vec<String>>,
    pub libs: Option<Vec<String>>,
    pub lib_dirs: Option<Vec<String>>,
    pub opt_level: Option<u8>,
    pub defines: Option<Vec<String>>,
    pub lto: Option<bool>,
    pub flags: Option<Vec<String>>,
    pub incremental: Option<bool>,
}

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
