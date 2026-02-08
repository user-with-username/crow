use crate::builder::kinds::compiler_kind::CompilerKind;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkerKind {
    Ld,
    Lld,
    Link,
    AppleLd,
    WasmLd,
    Gold,
    BpfLink,
    Unknown,
}

impl LinkerKind {
    pub fn detect(
        preferred: &Option<String>,
        preferred_kind: Option<LinkerKind>,
        compiler_kind: CompilerKind,
    ) -> Self {
        if let Some(kind) = preferred_kind {
            return kind;
        }

        if let Some(exe) = preferred.as_deref() {
            if let Ok(out) = Command::new(exe).output() {
                let info =
                    String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr);
                if info.contains("Microsoft (R) Incremental Linker")
                    || info.contains("Microsoft (R) Linker")
                {
                    return Self::Link;
                }
            }

            if let Ok(out) = Command::new(exe).arg("--version").output() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let info = stdout.to_string() + &stderr;

                if info.contains("LLD") {
                    return Self::Lld;
                }
                if info.contains("GNU gold") {
                    return Self::Gold;
                }
                if info.contains("GNU ld") {
                    return Self::Ld;
                }
                if info.contains("wasm-ld") {
                    return Self::WasmLd;
                }
                if info.contains("ld64") || info.contains("cctools") {
                    return Self::AppleLd;
                }
                if info.contains("BPF") && info.contains("ld") {
                    return Self::BpfLink;
                }
            }

            return Self::Unknown;
        }

        match compiler_kind {
            CompilerKind::Msvc => Self::Link,
            _ => Self::Unknown,
        }
    }

    pub fn default_executable(self) -> &'static str {
        match self {
            Self::Ld => "ld",
            Self::Lld => "ld.lld",
            Self::Link => "link",
            Self::AppleLd => "ld",
            Self::WasmLd => "wasm-ld",
            Self::Gold => "ld.gold",
            Self::BpfLink => "bpf-link",
            Self::Unknown => "ld",
        }
    }

    pub fn is_msvc(self) -> bool {
        matches!(self, Self::Link)
    }
}
