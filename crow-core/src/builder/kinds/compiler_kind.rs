use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerKind {
    Gcc,
    #[serde(alias = "g++")]
    Gpp,
    Clang,
    #[serde(alias = "clang++")]
    ClangPP,
    ClangCl,
    Msvc,
    Unknown,
}

impl CompilerKind {
    pub fn is_msvc(self) -> bool {
        matches!(self, Self::Msvc | Self::ClangCl)
    }

    pub fn is_gnu(self) -> bool {
        matches!(self, Self::Gcc | Self::Gpp | Self::Clang | Self::ClangPP)
    }
}
