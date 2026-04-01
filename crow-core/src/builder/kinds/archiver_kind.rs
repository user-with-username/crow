use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiverKind {
    Ar,     // Unix ar
    Lib,    // MSVC lib.exe
    LlvmAr, // llvm-ar
    Unknown,
}

impl ArchiverKind {
    pub fn is_msvc(self) -> bool {
        matches!(self, Self::Lib)
    }

    pub fn is_gnu(self) -> bool {
        matches!(self, Self::Ar | Self::LlvmAr)
    }
}
