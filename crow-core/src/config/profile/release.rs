use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct ReleaseProfile {
    #[default = "3"]
    pub opt_level: String,
    #[default = false]
    pub debug: bool,
    #[default = true]
    pub lto: bool,
    #[default = "fat"]
    pub lto_type: String,
    #[default = false]
    pub incremental: bool,
    #[default = 1]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = true]
    pub strip: bool,
}
