use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Debug, Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct DevProfile {
    #[default = "0"]
    pub opt_level: String,
    #[default = true]
    pub debug: bool,
    #[default = false]
    pub lto: bool,
    #[default = "regular"]
    pub lto_type: String,
    #[default = true]
    pub incremental: bool,
    #[default = 16]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = false]
    pub strip: bool,
}
