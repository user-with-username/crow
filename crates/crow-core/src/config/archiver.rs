use crate::builder::kinds::archiver_kind::ArchiverKind;
use crate::config::macros::config_enum;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum ArchiverConfig {
    Simple(ArchiverKind),
    Detailed {
        path: Option<String>,
        flags: Vec<String>,
        kind: Option<ArchiverKind>,
    },
}

impl Default for ArchiverConfig {
    fn default() -> Self {
        ArchiverConfig::Simple(ArchiverKind::Unknown)
    }
}

impl ArchiverConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            ArchiverConfig::Simple(_) => None,
            ArchiverConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn flags(&self) -> &[String] {
        match self {
            ArchiverConfig::Simple(_) => &[],
            ArchiverConfig::Detailed { flags, .. } => flags,
        }
    }

    pub fn kind(&self) -> ArchiverKind {
        match self {
            ArchiverConfig::Simple(kind) => *kind,
            ArchiverConfig::Detailed { kind: Some(k), .. } => *k,
            ArchiverConfig::Detailed { kind: None, .. } => ArchiverKind::Unknown,
        }
    }
}

config_enum!(ArchiverConfig, ArchiverKind);
