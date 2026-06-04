use target_lexicon::{Architecture, OperatingSystem, Triple};

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub triple: Triple,
    pub os: String,
    pub arch: String,
    pub vendor: String,
    pub env: String,
    pub pointer_width: u8,
    pub family: Option<String>,
}

impl TargetInfo {
    pub fn current() -> Self {
        let triple = Triple::host();
        let os = triple.operating_system.to_string();
        let arch = triple.architecture.to_string();
        let vendor = triple.vendor.to_string();
        let env = triple.environment.to_string();
        let pointer_width = match triple.architecture {
            Architecture::X86_64 | Architecture::Aarch64(_) => 64,
            Architecture::X86_32(_) | Architecture::Arm(_) => 32,
            _ => 64,
        };
        let family = match triple.operating_system {
            OperatingSystem::Windows => Some("windows".to_string()),
            OperatingSystem::Linux
            | OperatingSystem::MacOSX { .. }
            | OperatingSystem::Freebsd
            | OperatingSystem::Netbsd
            | OperatingSystem::Openbsd
            | OperatingSystem::Dragonfly => Some("unix".to_string()),
            _ => None,
        };
        Self { triple, os, arch, vendor, env, pointer_width, family }
    }

    pub fn is_windows(&self) -> bool { self.os == "windows" }
    pub fn is_unix(&self) -> bool { self.family.as_deref() == Some("unix") }
    pub fn is_macos(&self) -> bool { self.os == "macos" }
    pub fn is_linux(&self) -> bool { self.os == "linux" }
}