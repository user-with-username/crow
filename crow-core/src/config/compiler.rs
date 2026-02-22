use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::config::macros::config_enum;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum CompilerConfig {
    Simple(CompilerKind),
    Detailed {
        path: Option<String>,
        flags: Vec<String>,
        kind: Option<CompilerKind>,
    },
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig::Simple(CompilerKind::Unknown)
    }
}

impl CompilerConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            CompilerConfig::Simple(_) => None,
            CompilerConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn flags(&self) -> &[String] {
        match self {
            CompilerConfig::Simple(_) => &[],
            CompilerConfig::Detailed { flags, .. } => flags,
        }
    }

    pub fn kind(&self) -> CompilerKind {
        match self {
            CompilerConfig::Simple(kind) => *kind,
            CompilerConfig::Detailed { kind: Some(k), .. } => *k,
            CompilerConfig::Detailed { kind: None, .. } => CompilerKind::Unknown,
        }
    }
}

config_enum!(CompilerConfig, CompilerKind);