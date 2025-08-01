use crate::output_type::OutputType;
use crate::toolchain::toolchain_config::ToolchainConfig;

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
