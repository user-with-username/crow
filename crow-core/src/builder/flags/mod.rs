use crate::builder::compiler_kind::CompilerKind;

#[derive(Debug, Clone)]
pub enum Flag {
    OptimizationLevel(u8),
    OptimizeSize,
    OptimizeSpeed,
    NoOptimization,
    DependencyInfo(String),

    DebugInfo,
    DebugInfoFull,
    NoDebugInfo,

    AllWarnings,
    WarningsAsErrors,
    ExtraWarnings,
    NoWarnings,
    SpecificWarning(String),

    CxxStandard(String),
    CStandard(String),

    IncludePath(String),
    SystemIncludePath(String),

    Define(String, Option<String>),

    CompileOnly,
    OutputFile(String),
    ObjectOutput(String),

    LinkLibrary(String),
    LibraryPath(String),
    StaticLink,
    SharedLink,

    PositionIndependentCode,

    TargetArch(String),

    NoLogo,
    MultiThreadedDLL,
    ExceptionHandling,

    MachineDependent(String),
    FeatureFlag(String),
}

#[derive(Debug, Clone)]
pub struct Flags {
    compiler_kind: CompilerKind,
    flags: Vec<Flag>,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            compiler_kind: CompilerKind::detect(&None),
            flags: Vec::new(),
        }
    }

    pub fn optimization_level(&mut self, level: u8) -> &mut Self {
        self.flags.push(Flag::OptimizationLevel(level));
        self
    }

    pub fn optimize_size(&mut self) -> &mut Self {
        self.flags.push(Flag::OptimizeSize);
        self
    }

    pub fn optimize_speed(&mut self) -> &mut Self {
        self.flags.push(Flag::OptimizeSpeed);
        self
    }

    pub fn no_optimization(&mut self) -> &mut Self {
        self.flags.push(Flag::NoOptimization);
        self
    }

    pub fn debug_info(&mut self) -> &mut Self {
        self.flags.push(Flag::DebugInfo);
        self
    }

    pub fn debug_info_full(&mut self) -> &mut Self {
        self.flags.push(Flag::DebugInfoFull);
        self
    }

    pub fn no_debug_info(&mut self) -> &mut Self {
        self.flags.push(Flag::NoDebugInfo);
        self
    }

    pub fn all_warnings(&mut self) -> &mut Self {
        self.flags.push(Flag::AllWarnings);
        self
    }

    pub fn warnings_as_errors(&mut self) -> &mut Self {
        self.flags.push(Flag::WarningsAsErrors);
        self
    }

    pub fn extra_warnings(&mut self) -> &mut Self {
        self.flags.push(Flag::ExtraWarnings);
        self
    }

    pub fn no_warnings(&mut self) -> &mut Self {
        self.flags.push(Flag::NoWarnings);
        self
    }

    pub fn specific_warning(&mut self, warning: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::SpecificWarning(warning.into()));
        self
    }

    pub fn cxx_standard(&mut self, standard: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::CxxStandard(standard.into()));
        self
    }

    pub fn c_standard(&mut self, standard: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::CStandard(standard.into()));
        self
    }

    pub fn include_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::IncludePath(path.into()));
        self
    }

    pub fn system_include_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::SystemIncludePath(path.into()));
        self
    }

    pub fn define(
        &mut self,
        name: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> &mut Self {
        self.flags
            .push(Flag::Define(name.into(), value.map(|v| v.into())));
        self
    }

    pub fn dependency_info(&mut self, dep_file: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::DependencyInfo(dep_file.into()));
        self
    }

    pub fn compile_only(&mut self) -> &mut Self {
        self.flags.push(Flag::CompileOnly);
        self
    }

    pub fn output_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::OutputFile(path.into()));
        self
    }

    pub fn object_output(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::ObjectOutput(path.into()));
        self
    }

    pub fn link_library(&mut self, lib: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::LinkLibrary(lib.into()));
        self
    }

    pub fn library_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::LibraryPath(path.into()));
        self
    }

    pub fn static_link(&mut self) -> &mut Self {
        self.flags.push(Flag::StaticLink);
        self
    }

    pub fn shared_link(&mut self) -> &mut Self {
        self.flags.push(Flag::SharedLink);
        self
    }

    pub fn position_independent_code(&mut self) -> &mut Self {
        self.flags.push(Flag::PositionIndependentCode);
        self
    }

    pub fn target_arch(&mut self, arch: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::TargetArch(arch.into()));
        self
    }

    pub fn no_logo(&mut self) -> &mut Self {
        self.flags.push(Flag::NoLogo);
        self
    }

    pub fn multi_threaded_dll(&mut self) -> &mut Self {
        self.flags.push(Flag::MultiThreadedDLL);
        self
    }

    pub fn exception_handling(&mut self) -> &mut Self {
        self.flags.push(Flag::ExceptionHandling);
        self
    }

    pub fn add_raw(&mut self, raw_flag: impl AsRef<str>) -> &mut Self {
        let flag_str = raw_flag.as_ref();
        if let Some(parsed) = Self::parse_raw(flag_str) {
            self.flags.push(parsed);
        } else {
            self.flags
                .push(Flag::MachineDependent(flag_str.to_string()));
        }
        self
    }

    pub fn build(&self) -> Vec<String> {
        self.flags
            .iter()
            .flat_map(|f| Self::convert_flag(f, &self.compiler_kind))
            .collect()
    }

    fn parse_raw(flag: &str) -> Option<Flag> {
        let flag = flag.trim();

        if flag.starts_with("-O") || flag.starts_with("/O") {
            let level = flag.trim_start_matches("-O").trim_start_matches("/O");
            return match level {
                "0" | "d" => Some(Flag::NoOptimization),
                "1" | "s" => Some(Flag::OptimizationLevel(1)),
                "2" => Some(Flag::OptimizationLevel(2)),
                "3" | "x" | "fast" => Some(Flag::OptimizationLevel(3)),
                _ => None,
            };
        }

        if flag.starts_with("-I") {
            return Some(Flag::IncludePath(flag[2..].to_string()));
        }
        if flag.starts_with("/I") {
            return Some(Flag::IncludePath(flag[2..].to_string()));
        }

        if flag.starts_with("-D") || flag.starts_with("/D") {
            let def = flag[2..].to_string();
            if let Some(pos) = def.find('=') {
                return Some(Flag::Define(
                    def[..pos].to_string(),
                    Some(def[pos + 1..].to_string()),
                ));
            } else {
                return Some(Flag::Define(def, None));
            }
        }

        if flag.starts_with("-l") {
            return Some(Flag::LinkLibrary(flag[2..].to_string()));
        }
        if flag.starts_with("-L") {
            return Some(Flag::LibraryPath(flag[2..].to_string()));
        }
        if flag.starts_with("/LIBPATH:") {
            return Some(Flag::LibraryPath(flag[9..].to_string()));
        }

        if flag.starts_with("-o") && flag.len() > 2 {
            return Some(Flag::OutputFile(flag[2..].to_string()));
        }
        if flag.starts_with("/Fe") {
            return Some(Flag::OutputFile(flag[3..].to_string()));
        }
        if flag.starts_with("/Fo") {
            return Some(Flag::ObjectOutput(flag[3..].to_string()));
        }

        match flag {
            "-c" | "/c" => Some(Flag::CompileOnly),
            "-g" | "/Zi" | "/Z7" => Some(Flag::DebugInfo),
            "-g3" => Some(Flag::DebugInfoFull),
            "-Wall" | "/W4" => Some(Flag::AllWarnings),
            "-Werror" | "/WX" => Some(Flag::WarningsAsErrors),
            "-Wextra" => Some(Flag::ExtraWarnings),
            "-w" | "/w" => Some(Flag::NoWarnings),
            "-fPIC" => Some(Flag::PositionIndependentCode),
            "-static" | "/MT" => Some(Flag::StaticLink),
            "-shared" | "/LD" => Some(Flag::SharedLink),
            "/nologo" => Some(Flag::NoLogo),
            "/MD" => Some(Flag::MultiThreadedDLL),
            "/EHsc" => Some(Flag::ExceptionHandling),
            _ => {
                if flag.starts_with("-std=") {
                    return Some(Flag::CxxStandard(flag[5..].to_string()));
                }
                if flag.starts_with("/std:") {
                    return Some(Flag::CxxStandard(flag[5..].to_string()));
                }
                if flag.starts_with("-m") {
                    return Some(Flag::MachineDependent(flag[2..].to_string()));
                }
                if flag.starts_with("-f") {
                    return Some(Flag::FeatureFlag(flag[2..].to_string()));
                }
                None
            }
        }
    }

    fn convert_flag(flag: &Flag, compiler_kind: &CompilerKind) -> Vec<String> {
        if compiler_kind.is_msvc() {
            Self::to_msvc(flag)
        } else {
            Self::to_gcc_like(flag)
        }
    }

    fn to_gcc_like(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::OptimizationLevel(n) => vec![format!("-O{}", n)],
            Flag::OptimizeSize => vec!["-Os".to_string()],
            Flag::OptimizeSpeed => vec!["-O3".to_string()],
            Flag::NoOptimization => vec!["-O0".to_string()],
            Flag::DependencyInfo(path) => vec!["-MD".to_string(), "-MF".to_string(), path.clone()],

            Flag::DebugInfo => vec!["-g".to_string()],
            Flag::DebugInfoFull => vec!["-g3".to_string()],
            Flag::NoDebugInfo => vec![],

            Flag::AllWarnings => vec!["-Wall".to_string()],
            Flag::WarningsAsErrors => vec!["-Werror".to_string()],
            Flag::ExtraWarnings => vec!["-Wextra".to_string()],
            Flag::NoWarnings => vec!["-w".to_string()],
            Flag::SpecificWarning(w) => vec![format!("-W{}", w)],

            Flag::CxxStandard(std) => vec![format!("-std={}", std)],
            Flag::CStandard(std) => vec![format!("-std={}", std)],

            Flag::IncludePath(path) => vec![format!("-I{}", path)],
            Flag::SystemIncludePath(path) => vec!["-isystem".to_string(), path.clone()],

            Flag::Define(name, val) => match val {
                Some(v) => vec![format!("-D{}={}", name, v)],
                None => vec![format!("-D{}", name)],
            },

            Flag::CompileOnly => vec!["-c".to_string()],
            Flag::OutputFile(path) => vec!["-o".to_string(), path.clone()],
            Flag::ObjectOutput(path) => vec!["-o".to_string(), path.clone()],

            Flag::LinkLibrary(lib) => vec![format!("-l{}", lib)],
            Flag::LibraryPath(path) => vec![format!("-L{}", path)],
            Flag::StaticLink => vec!["-static".to_string()],
            Flag::SharedLink => vec!["-shared".to_string()],

            Flag::PositionIndependentCode => vec!["-fPIC".to_string()],
            Flag::TargetArch(arch) => vec![format!("-march={}", arch)],

            Flag::NoLogo => vec![],
            Flag::MultiThreadedDLL => vec!["-pthread".to_string()],
            Flag::ExceptionHandling => vec![],

            Flag::MachineDependent(f) => {
                if f.starts_with('/') {
                    vec![f.replacen('/', "-", 1)]
                } else if f.starts_with('-') {
                    vec![f.clone()]
                } else {
                    vec![format!("-{}", f)]
                }
            }
            Flag::FeatureFlag(f) => vec![format!("-f{}", f)],
        }
    }

    fn to_msvc(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::OptimizationLevel(n) => match n {
                0 => vec!["/Od".to_string()],
                1 => vec!["/O1".to_string()],
                2 => vec!["/O2".to_string()],
                3 => vec!["/Ox".to_string()],
                _ => vec!["/O2".to_string()],
            },
            Flag::OptimizeSize => vec!["/O1".to_string()],
            Flag::OptimizeSpeed => vec!["/O2".to_string()],
            Flag::NoOptimization => vec!["/Od".to_string()],
            Flag::DependencyInfo(path) => vec!["/showIncludes".to_string(), format!("/FD{}", path)],

            Flag::DebugInfo => vec!["/Zi".to_string()],
            Flag::DebugInfoFull => vec!["/Zi".to_string()],
            Flag::NoDebugInfo => vec![],

            Flag::AllWarnings => vec!["/W4".to_string()],
            Flag::WarningsAsErrors => vec!["/WX".to_string()],
            Flag::ExtraWarnings => vec!["/W4".to_string()],
            Flag::NoWarnings => vec!["/w".to_string()],
            Flag::SpecificWarning(w) => vec![format!("/W{}", w)],

            Flag::CxxStandard(std) => vec![format!("/std:{}", std)],
            Flag::CStandard(std) => vec![format!("/std:{}", std)],

            Flag::IncludePath(path) => vec![format!("/I{}", path)],
            Flag::SystemIncludePath(path) => vec![format!("/I{}", path)],

            Flag::Define(name, val) => match val {
                Some(v) => vec![format!("/D{}={}", name, v)],
                None => vec![format!("/D{}", name)],
            },

            Flag::CompileOnly => vec!["/c".to_string()],
            Flag::OutputFile(path) => vec![format!("/Fe{}", path)],
            Flag::ObjectOutput(path) => vec![format!("/Fo{}", path)],

            Flag::LinkLibrary(lib) => {
                if lib.ends_with(".lib") {
                    vec![lib.clone()]
                } else {
                    vec![format!("{}.lib", lib)]
                }
            }
            Flag::LibraryPath(path) => vec![format!("/LIBPATH:{}", path)],
            Flag::StaticLink => vec!["/MT".to_string()],
            Flag::SharedLink => vec!["/LD".to_string()],

            Flag::PositionIndependentCode => vec![],
            Flag::TargetArch(arch) => vec![format!("/arch:{}", arch)],

            Flag::NoLogo => vec!["/nologo".to_string()],
            Flag::MultiThreadedDLL => vec!["/MD".to_string()],
            Flag::ExceptionHandling => vec!["/EHsc".to_string()],

            Flag::MachineDependent(f) => {
                if f.starts_with('-') {
                    vec![f.replacen('-', "/", 1)]
                } else if f.starts_with('/') {
                    vec![f.clone()]
                } else {
                    vec![format!("/{}", f)]
                }
            }
            Flag::FeatureFlag(_) => vec![],
        }
    }
}
