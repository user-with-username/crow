use serde::{Deserialize, Serialize};

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
