use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ToolchainHooks {
    pub pre_execute: Option<Vec<String>>,
    pub post_execute: Option<Vec<String>>,
}