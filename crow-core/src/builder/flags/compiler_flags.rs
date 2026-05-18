use crate::builder::kinds::compiler_kind::CompilerKind;
use crow_utils::{fix_msvc_path, normalize_path};

use super::flag::Flag;

#[derive(Debug, Clone)]
pub struct CompilerFlags {
    compiler_kind: CompilerKind,
    flags: Vec<Flag>,
}

impl CompilerFlags {
    pub fn new(compiler_kind: CompilerKind) -> Self {
        Self {
            compiler_kind,
            flags: Vec::new(),
        }
    }

    pub fn with_compiler(mut self, compiler_kind: CompilerKind) -> Self {
        self.compiler_kind = compiler_kind;
        self
    }

    pub fn get_compiler_kind(&self) -> &CompilerKind {
        &self.compiler_kind
    }

    pub fn standard_flags(&mut self) -> &mut Self {
        if self.compiler_kind.is_msvc() {
            self.no_logo();
            self.exception_handling();
            self.all_warnings();
            self.add_feature_flag("utf-8");
        }
        self
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
        let dep_file = dep_file.into();
        if self.compiler_kind.is_msvc() {
            self.show_includes();
            self.flags.push(Flag::SourceDependencies(dep_file));
        } else {
            self.flags.push(Flag::DependencyInfo(dep_file));
        }
        self
    }

    pub fn show_includes(&mut self) -> &mut Self {
        self.flags.push(Flag::ShowIncludes);
        self
    }

    pub fn source_dependencies(&mut self, dep_file: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::SourceDependencies(dep_file.into()));
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
        let mut path = path.into();
        if self.compiler_kind.is_msvc() {
            if path.ends_with(".o") {
                path = path.replacen(".o", ".obj", 1);
            }
        }
        self.flags.push(Flag::ObjectOutput(path));
        self
    }

    pub fn position_independent_code(&mut self) -> &mut Self {
        self.flags.push(Flag::PositionIndependentCode);
        self
    }

    pub fn link_time_optimization(&mut self) -> &mut Self {
        self.flags.push(Flag::LinkTimeOptimization);
        self
    }

    pub fn no_link_time_optimization(&mut self) -> &mut Self {
        self.flags.push(Flag::NoLinkTimeOptimization);
        self
    }

    pub fn thin_lto(&mut self) -> &mut Self {
        self.flags.push(Flag::ThinLTO);
        self
    }

    pub fn fat_lto(&mut self) -> &mut Self {
        self.flags.push(Flag::FatLTO);
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

    pub fn multi_threaded_dll_debug(&mut self) -> &mut Self {
        self.flags.push(Flag::MultiThreadedDLLDebug);
        self
    }

    pub fn exception_handling(&mut self) -> &mut Self {
        self.flags.push(Flag::ExceptionHandling);
        self
    }

    pub fn program_database(&mut self, pdb_path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::ProgramDatabase(pdb_path.into()));
        self
    }

    pub fn debug_type(&mut self, debug_type: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::DebugType(debug_type.into()));
        self
    }

    pub fn add_raw(&mut self, raw_flag: impl AsRef<str>) -> &mut Self {
        self.flags.push(Flag::Raw(raw_flag.as_ref().to_string()));
        self
    }

    pub fn import_library(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::ImportLibrary(path.into()));
        self
    }

    pub fn add_machine_dependent(&mut self, flag: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::MachineDependent(flag.into()));
        self
    }

    pub fn add_feature_flag(&mut self, flag: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::FeatureFlag(flag.into()));
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut flags: Vec<String> = self
            .flags
            .iter()
            .flat_map(|f| Self::convert_flag(f, &self.compiler_kind))
            .filter(|s| !s.is_empty())
            .collect();

        if self.compiler_kind.is_msvc() {
            if let Some(pos) = flags.iter().position(|f| f == "/c") {
                if pos != 0 {
                    let c_flag = flags.remove(pos);
                    flags.insert(0, c_flag);
                }
            }
        }

        flags
    }

    fn convert_flag(flag: &Flag, compiler_kind: &CompilerKind) -> Vec<String> {
        match flag {
            Flag::Raw(f) => vec![f.clone()],
            _ => match compiler_kind {
                CompilerKind::ClangCl => match flag {
                    Flag::LinkTimeOptimization
                    | Flag::NoLinkTimeOptimization
                    | Flag::ThinLTO
                    | Flag::FatLTO => Self::to_gcc_like(flag),
                    _ => Self::to_msvc(flag),
                },
                _ if compiler_kind.is_msvc() => Self::to_msvc(flag),
                _ => Self::to_gcc_like(flag),
            },
        }
    }

    fn to_gcc_like(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::Raw(_) => unreachable!(),
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
            Flag::ImportLibrary(path) => {
                vec![format!("-Wl,--out-implib,{}", normalize_path(path))]
            }

            Flag::CxxStandard(std) => {
                let std = if std.chars().all(|c| c.is_numeric()) {
                    format!("c++{}", std)
                } else {
                    std.clone()
                };
                vec![format!("-std={}", std)]
            }
            Flag::CStandard(std) => {
                let std = if std.chars().all(|c| c.is_numeric()) {
                    format!("c{}", std)
                } else {
                    std.clone()
                };
                vec![format!("-std={}", std)]
            }

            Flag::IncludePath(path) => vec![format!("-I{}", normalize_path(path))],
            Flag::SystemIncludePath(path) => vec!["-isystem".to_string(), normalize_path(path)],

            Flag::Define(name, val) => match val {
                Some(v) => vec![format!("-D{}={}", name, v)],
                None => vec![format!("-D{}", name)],
            },

            Flag::CompileOnly => vec!["-c".to_string()],
            Flag::OutputFile(path) => vec![format!("/Fe{}", normalize_path(path))],
            Flag::ObjectOutput(path) => vec!["-o".to_string(), normalize_path(path)],

            Flag::LinkLibrary(_) => vec![],
            Flag::LibraryPath(_) => vec![],
            Flag::StaticLink => vec![],
            Flag::SharedLink => vec![],

            Flag::PositionIndependentCode => vec!["-fPIC".to_string()],
            Flag::LinkTimeOptimization => vec!["-flto".to_string()],
            Flag::NoLinkTimeOptimization => vec!["-fno-lto".to_string()],
            Flag::ThinLTO => vec!["-flto=thin".to_string()],
            Flag::FatLTO => vec!["-flto=full".to_string()],
            Flag::TargetArch(arch) => vec![format!("-march={}", arch)],

            Flag::NoLogo => vec![],
            Flag::MultiThreadedDLL => vec!["-pthread".to_string()],
            Flag::MultiThreadedDLLDebug => vec!["-pthread".to_string()],
            Flag::ExceptionHandling => vec![],

            Flag::MachineDependent(f) => {
                if f.starts_with('/') {
                    vec![f.replacen('/', "-", 1)]
                } else if f.starts_with('-') {
                    vec![f.clone()]
                } else {
                    vec![format!("-m{}", f)]
                }
            }
            Flag::FeatureFlag(f) => vec![format!("-f{}", f)],

            Flag::ShowIncludes => vec![],
            Flag::SourceDependencies(_) => vec![],

            Flag::ProgramDatabase(_) => vec![],
            Flag::LinkProgramDatabase(_) => vec![],
            Flag::DebugType(_) => vec![],
        }
    }

    fn to_msvc(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::Raw(_) => unreachable!(),
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

            Flag::DependencyInfo(_) => vec![],

            Flag::DebugInfo => vec!["/Zi".to_string()],
            Flag::DebugInfoFull => vec!["/Z7".to_string()],
            Flag::NoDebugInfo => vec![],

            Flag::AllWarnings => vec!["/W4".to_string()],
            Flag::WarningsAsErrors => vec!["/WX".to_string()],
            Flag::ExtraWarnings => vec!["/W4".to_string()],
            Flag::NoWarnings => vec!["/w".to_string()],
            Flag::SpecificWarning(w) => vec![format!("/W{}", w)],

            Flag::CxxStandard(std) => {
                let std = if std.chars().all(|c| c.is_numeric()) {
                    format!("c++{}", std)
                } else {
                    std.clone()
                };
                vec![format!("/std:{}", std)]
            }
            Flag::CStandard(std) => {
                let std = if std.chars().all(|c| c.is_numeric()) {
                    format!("c{}", std)
                } else {
                    std.clone()
                };
                vec![format!("/std:{}", std)]
            }

            Flag::IncludePath(path) => vec![format!("/I{}", path)],
            Flag::SystemIncludePath(path) => vec![format!("/I{}", path)],

            Flag::Define(name, val) => match val {
                Some(v) => vec![format!("/D{}={}", name, v)],
                None => vec![format!("/D{}", name)],
            },
            Flag::ImportLibrary(path) => {
                vec![format!("/IMPLIB:{}", normalize_path(path))]
            }

            Flag::CompileOnly => vec!["/c".to_string()],
            Flag::OutputFile(path) => vec![format!("/Fe{}", normalize_path(path))],
            Flag::ObjectOutput(path) => {
                let normalized_path = fix_msvc_path(path, Some(".obj"));
                vec![format!("/Fo:{}", normalized_path)]
            }

            Flag::LinkLibrary(_) => vec![],
            Flag::LibraryPath(_) => vec![],
            Flag::StaticLink => vec![],
            Flag::SharedLink => vec![],

            Flag::PositionIndependentCode => vec![],
            Flag::LinkTimeOptimization => vec!["/GL".to_string()],
            Flag::NoLinkTimeOptimization => vec![],
            Flag::ThinLTO => vec!["/GL".to_string()],
            Flag::FatLTO => vec!["/GL".to_string()],
            Flag::TargetArch(arch) => vec![format!("/arch:{}", arch)],

            Flag::NoLogo => vec!["/nologo".to_string()],
            Flag::MultiThreadedDLL => vec!["/MD".to_string()],
            Flag::MultiThreadedDLLDebug => vec!["/MDd".to_string()],
            Flag::ExceptionHandling => vec!["/EHsc".to_string()],

            Flag::ShowIncludes => vec!["/showIncludes".to_string()],
            Flag::SourceDependencies(_) => vec![],

            Flag::MachineDependent(f) => {
                if f.starts_with('-') {
                    vec![f.replacen('-', "/", 1)]
                } else if f.starts_with('/') {
                    vec![f.clone()]
                } else {
                    vec![format!("/{}", f)]
                }
            }
            Flag::FeatureFlag(f) => vec![format!("/{}", f)],

            Flag::ProgramDatabase(pdb_path) => vec![format!("/Fd{}", normalize_path(pdb_path))],
            Flag::LinkProgramDatabase(_) => vec![],
            Flag::DebugType(debug_type) => match debug_type.as_str() {
                "7" | "Z7" => vec!["/Z7".to_string()],
                "i" | "Zi" => vec!["/Zi".to_string()],
                "I" | "ZI" => vec!["/ZI".to_string()],
                "1" | "Z1" => vec!["/Z1".to_string()],
                _ => vec![format!("/Z{}", debug_type)],
            },
        }
    }
}
