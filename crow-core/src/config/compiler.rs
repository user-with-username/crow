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
        CompilerConfig::Simple(CompilerKind::detect(None, None))
    }
}

impl CompilerConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            CompilerConfig::Simple(_) => None,
            CompilerConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn executable(&self) -> &str {
        match self {
            CompilerConfig::Simple(kind) => kind.default_executable(),
            CompilerConfig::Detailed { path: Some(p), .. } => p,
            CompilerConfig::Detailed {
                path: None,
                kind: Some(k),
                ..
            } => k.default_executable(),
            CompilerConfig::Detailed {
                path: None,
                kind: None,
                ..
            } => CompilerKind::detect(None, None).default_executable(),
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
            CompilerConfig::Detailed { kind: None, .. } => {
                if let Some(path) = self.path() {
                    CompilerKind::detect(Some(path.clone()), None)
                } else {
                    CompilerKind::detect(None, None)
                }
            }
        }
    }
}

config_enum!(CompilerConfig, CompilerKind);
