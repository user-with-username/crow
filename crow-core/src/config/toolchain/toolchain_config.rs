use crate::toolchain::toolchain_hooks::ToolchainHooks;
use crate::toolchain::toolchain_override::ToolchainOverride;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ToolchainConfig {
    pub compiler: String,
    pub compiler_flags: Vec<String>,
    pub linker: String,
    pub linker_flags: Vec<String>,
    pub archiver: String,
    pub archiver_flags: Vec<String>,
    pub hooks: ToolchainHooks,
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
        if std::process::Command::new("clang")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(("clang".to_string(), "clang".to_string()));
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

    pub fn merge(&self, override_config: Option<&ToolchainOverride>) -> Self {
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
            hooks: ToolchainHooks {
                pre_execute: ov.hooks.pre_execute.clone().or(self.hooks.pre_execute.clone()),
                post_execute: ov.hooks.post_execute.clone().or(self.hooks.post_execute.clone()),
            },
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
            hooks: ToolchainHooks {
                pre_execute: None,
                post_execute: None,
            },
        }
    }
}