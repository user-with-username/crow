use crate::builder::kinds::compiler_kind::CompilerKind;
use crow_utils::normalize_path;

use super::flag::Flag;

#[derive(Debug, Clone)]
pub struct LinkerFlags {
    compiler_kind: CompilerKind,
    flags: Vec<Flag>,
}

impl LinkerFlags {
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

    pub fn use_lld(&mut self) -> &mut Self {
        if self.compiler_kind != CompilerKind::Msvc {
            self.add_raw("-fuse-ld=lld");
        }
        self
    }

    pub fn get_compiler_kind(&self) -> &CompilerKind {
        &self.compiler_kind
    }

    pub fn standard_flags(&mut self) -> &mut Self {
        if self.compiler_kind.is_msvc() && self.compiler_kind != CompilerKind::ClangCl {
            self.no_logo();
        }
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

    pub fn output_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::OutputFile(path.into()));
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

    pub fn link_program_database(&mut self, pdb_path: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::LinkProgramDatabase(pdb_path.into()));
        self
    }

    pub fn add_raw(&mut self, raw_flag: impl AsRef<str>) -> &mut Self {
        self.flags.push(Flag::Raw(raw_flag.as_ref().to_string()));
        self
    }

    pub fn add_machine_dependent(&mut self, flag: impl Into<String>) -> &mut Self {
        self.flags.push(Flag::MachineDependent(flag.into()));
        self
    }

    pub fn build(&self) -> Vec<String> {
        let result = self
            .flags
            .iter()
            .flat_map(|f| Self::convert_flag(f, &self.compiler_kind))
            .filter(|s| !s.is_empty())
            .collect();
        result
    }

    fn convert_flag(flag: &Flag, compiler_kind: &CompilerKind) -> Vec<String> {
        match flag {
            Flag::Raw(f) => vec![f.clone()],
            _ => {
                if compiler_kind.is_msvc() && *compiler_kind != CompilerKind::ClangCl {
                    Self::to_msvc(flag)
                } else {
                    Self::to_gcc_like(flag)
                }
            }
        }
    }

    fn to_gcc_like(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::Raw(_) => unreachable!(),
            Flag::DebugInfo => vec!["-g".to_string()],
            Flag::DebugInfoFull => vec!["-g3".to_string()],
            Flag::NoDebugInfo => vec![],

            Flag::OutputFile(path) => vec!["-o".to_string(), normalize_path(path)],

            Flag::LinkLibrary(lib) => {
                if lib.ends_with(".lib") || lib.ends_with(".a") {
                    vec![lib.clone()]
                } else {
                    vec![format!("-l{}", lib)]
                }
            }
            Flag::LibraryPath(path) => vec![format!("-L{}", normalize_path(path))],
            Flag::StaticLink => vec!["-static".to_string()],
            Flag::SharedLink => vec!["-shared".to_string()],

            Flag::LinkTimeOptimization => vec!["-flto".to_string()],
            Flag::NoLinkTimeOptimization => vec!["-fno-lto".to_string()],
            Flag::ThinLTO => vec!["-flto=thin".to_string()],
            Flag::FatLTO => vec!["-flto=full".to_string()],

            Flag::TargetArch(arch) => vec![format!("-march={}", arch)],

            Flag::NoLogo => vec![],

            Flag::MachineDependent(f) => {
                if f.starts_with('/') {
                    vec![f.replacen('/', "-", 1)]
                } else if f.starts_with('-') {
                    vec![f.clone()]
                } else {
                    vec![format!("-m{}", f)]
                }
            }

            Flag::LinkProgramDatabase(_) => vec![],

            _ => vec![],
        }
    }

    fn to_msvc(flag: &Flag) -> Vec<String> {
        match flag {
            Flag::Raw(_) => unreachable!(),
            Flag::DebugInfo => vec!["/DEBUG".to_string()],
            Flag::DebugInfoFull => vec!["/Z7".to_string()],
            Flag::NoDebugInfo => vec![],

            Flag::OutputFile(path) => vec![format!("/OUT:{}", normalize_path(path))],

            Flag::LinkLibrary(lib) => {
                if lib.ends_with(".lib") {
                    vec![lib.clone()]
                } else {
                    vec![format!("{}.lib", lib)]
                }
            }
            Flag::LibraryPath(path) => vec![format!("/LIBPATH:{}", normalize_path(path))],
            Flag::StaticLink => vec!["/MT".to_string()],
            Flag::SharedLink => vec!["/LD".to_string()],

            Flag::LinkTimeOptimization => vec!["/LTCG".to_string()],
            Flag::NoLinkTimeOptimization => vec!["/LTCG:OFF".to_string()],
            Flag::ThinLTO => vec!["/LTCG".to_string()],
            Flag::FatLTO => vec!["/LTCG".to_string()],
            Flag::TargetArch(arch) => vec![format!("/arch:{}", arch)],

            Flag::NoLogo => vec!["/nologo".to_string()],

            Flag::MachineDependent(f) => {
                if f.starts_with('-') {
                    vec![f.replacen('-', "/", 1)]
                } else if f.starts_with('/') {
                    vec![f.clone()]
                } else {
                    vec![format!("/{}", f)]
                }
            }

            Flag::LinkProgramDatabase(pdb_path) => {
                vec![format!("/PDB:{}", normalize_path(pdb_path))]
            }

            _ => vec![],
        }
    }
}
