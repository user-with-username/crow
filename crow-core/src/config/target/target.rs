use crate::output_type::OutputType;
use crate::toolchain::toolchain_override::ToolchainOverride;
use serde::{Deserialize, Serialize};

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
