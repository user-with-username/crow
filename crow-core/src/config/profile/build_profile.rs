use serde::{Deserialize, Serialize};

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
    pub fn default_debug() -> Self {
        BuildProfile {
            opt_level: 0,
            defines: vec!["DEBUG".to_string()],
            lto: false,
            flags: vec!["-g".to_string()],
            incremental: true,
        }
    }

    pub fn default_release() -> Self {
        BuildProfile {
            opt_level: 3,
            defines: vec!["NDEBUG".to_string()],
            lto: true,
            flags: vec!["-O3".to_string()],
            incremental: false,
        }
    }
}
