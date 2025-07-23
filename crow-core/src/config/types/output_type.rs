use serde::{Deserialize, Serialize};

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
