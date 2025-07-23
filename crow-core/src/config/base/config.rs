use crate::PackageConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    dependency::Dependency, profile::BuildProfile, target::Target,
    toolchain::toolchain_config::ToolchainConfig,
};

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

    fn generate_default_profiles_map() -> HashMap<String, BuildProfile> {
        let mut profiles = HashMap::new();
        profiles.insert("debug".to_string(), BuildProfile::default_debug());
        profiles.insert("release".to_string(), BuildProfile::default_release());
        profiles
    }
}
