use crate::builder::kinds::compiler_kind::CompilerKind;
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

    pub fn kind(&self, compiler_kind: CompilerKind) -> LinkerKind {
        match self {
            LinkerConfig::Simple(kind) => {
                if *kind == LinkerKind::Unknown {
                    LinkerKind::detect(self.path().cloned(), None, compiler_kind)
                } else {
                    *kind
                }
            }
            LinkerConfig::Detailed { kind: Some(k), .. } => *k,
            LinkerConfig::Detailed {
                kind: None, path, ..
            } => LinkerKind::detect(path.clone(), None, compiler_kind),
        }
    }

    pub fn executable(&self, compiler_kind: CompilerKind) -> &str {
        match self {
            LinkerConfig::Simple(kind) => {
                if *kind == LinkerKind::Unknown {
                    self.kind(compiler_kind).default_executable()
                } else {
                    kind.default_executable()
                }
            }
            LinkerConfig::Detailed { path: Some(p), .. } => p,
            LinkerConfig::Detailed {
                path: None,
                kind: Some(k),
                ..
            } => k.default_executable(),
            LinkerConfig::Detailed {
                path: None,
                kind: None,
                ..
            } => self.kind(compiler_kind).default_executable(),
        }
    }
}

config_enum!(LinkerConfig, LinkerKind);
