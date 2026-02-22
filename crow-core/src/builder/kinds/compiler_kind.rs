use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerKind {
    Gcc,
    #[serde(alias = "g++", alias = "gpp")]
    Gpp,
    Clang,
    #[serde(alias = "clang++", alias = "clangpp", alias = "clang_p_p")]
    ClangPP,
    #[serde(alias = "cl", alias = "msvc-cl")]
    Msvc,
    Unknown,
}

impl CompilerKind {
    pub fn is_msvc(self) -> bool {
        matches!(self, Self::Msvc)
    }
}
