use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerKind {
    Gcc,
    Gpp,
    Clang,
    ClangPP,
    Msvc,
}

impl CompilerKind {
    pub fn detect(preferred: &Option<String>) -> Self {
        let exe = preferred.as_deref().unwrap_or("g++");

        if let Ok(out) = Command::new(exe).output() {
            let info = String::from_utf8_lossy(&out.stdout)
                + String::from_utf8_lossy(&out.stderr);
            if info.contains("Microsoft (R) C/C++") {
                return Self::Msvc;
            }
        }

        if let Ok(out) = Command::new(exe)
            .args(["-dM", "-E", "-"])
            .stdin(Stdio::null())
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("__clang__") {
                return if exe.contains("++") { Self::ClangPP } else { Self::Clang };
            }
            if stdout.contains("__GNUC__") {
                return if exe.contains("++") { Self::Gpp } else { Self::Gcc };
            }
        }

        Self::Gpp
    }

    pub fn default_executable(self) -> &'static str {
        match self {
            Self::Gcc => "gcc",
            Self::Gpp => "g++",
            Self::Clang => "clang",
            Self::ClangPP => "clang++",
            Self::Msvc => "cl",
        }
    }

    pub fn is_msvc(self) -> bool {
        matches!(self, Self::Msvc)
    }
}
