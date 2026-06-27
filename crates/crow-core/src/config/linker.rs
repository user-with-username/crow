use crate::builder::kinds::linker_kind::LinkerKind;
use crate::config::macros::config_enum;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum LinkerConfig {
    Simple(LinkerKind),
    Detailed {
        path: Option<String>,
        flags: Vec<String>,
        kind: Option<LinkerKind>,
    },
}

impl Default for LinkerConfig {
    fn default() -> Self {
        LinkerConfig::Simple(LinkerKind::Unknown)
    }
}

impl LinkerConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            LinkerConfig::Simple(_) => None,
            LinkerConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn flags(&self) -> &[String] {
        match self {
            LinkerConfig::Simple(_) => &[],
            LinkerConfig::Detailed { flags, .. } => flags,
        }
    }

    pub fn kind(&self) -> LinkerKind {
        match self {
            LinkerConfig::Simple(kind) => *kind,
            LinkerConfig::Detailed { kind: Some(k), .. } => *k,
            LinkerConfig::Detailed { kind: None, .. } => LinkerKind::Unknown,
        }
    }
}

config_enum!(LinkerConfig, LinkerKind);
