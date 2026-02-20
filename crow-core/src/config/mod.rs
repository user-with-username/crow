use serde::de::{self, Deserializer, MapAccess};
use serde::Deserialize;
use smart_default::SmartDefault;
use std::fmt;
use std::path::PathBuf;

use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;


#[derive(Deserialize)]
struct DetailedConfig<T> {
    path: Option<String>,
    #[serde(default)]
    flags: Vec<String>,
    kind: Option<T>,
}


macro_rules! config_enum {
    ($name:ident, $kind:ty) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct ConfigVisitor;

                impl<'de> de::Visitor<'de> for ConfigVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a string or a struct with path, flags, and kind")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: de::Error,
                    {
                        let kind = <$kind>::deserialize(de::value::StrDeserializer::new(value))?;
                        Ok($name::Simple(kind))
                    }

                    fn visit_map<M>(self, map: M) -> Result<$name, M::Error>
                    where
                        M: MapAccess<'de>,
                    {
                        let detailed =
                            DetailedConfig::<$kind>::deserialize(
                                de::value::MapAccessDeserializer::new(map)
                            )?;
                        Ok($name::Detailed {
                            path: detailed.path,
                            flags: detailed.flags,
                            kind: detailed.kind,
                        })
                    }
                }

                deserializer.deserialize_any(ConfigVisitor)
            }
        }
    };
}

#[derive(Deserialize)]
pub struct CrowConfig {
    pub package: Package,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub profile: Profiles,
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}


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
            LinkerConfig::Detailed { kind: None, path, .. } => {
                LinkerKind::detect(path.clone(), None, compiler_kind)
            }
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
            LinkerConfig::Detailed { path: None, kind: Some(k), .. } => k.default_executable(),
            LinkerConfig::Detailed { path: None, kind: None, .. } => {
                self.kind(compiler_kind).default_executable()
            }
        }
    }
}


config_enum!(LinkerConfig, LinkerKind);


#[derive(Debug, Clone)]
pub enum CompilerConfig {
    Simple(CompilerKind),
    Detailed {
        path: Option<String>,
        flags: Vec<String>,
        kind: Option<CompilerKind>,
    },
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig::Simple(CompilerKind::detect(None, None))
    }
}

impl CompilerConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            CompilerConfig::Simple(_) => None,
            CompilerConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn executable(&self) -> &str {
        match self {
            CompilerConfig::Simple(kind) => kind.default_executable(),
            CompilerConfig::Detailed { path: Some(p), .. } => p,
            CompilerConfig::Detailed {
                path: None,
                kind: Some(k),
                ..
            } => k.default_executable(),
            CompilerConfig::Detailed {
                path: None,
                kind: None,
                ..
            } => CompilerKind::detect(None, None).default_executable(),
        }
    }

    pub fn flags(&self) -> &[String] {
        match self {
            CompilerConfig::Simple(_) => &[],
            CompilerConfig::Detailed { flags, .. } => flags,
        }
    }

    pub fn kind(&self) -> CompilerKind {
        match self {
            CompilerConfig::Simple(kind) => *kind,
            CompilerConfig::Detailed { kind: Some(k), .. } => *k,
            CompilerConfig::Detailed { kind: None, .. } => {
                if let Some(path) = self.path() {
                    CompilerKind::detect(Some(path.clone()), None)
                } else {
                    CompilerKind::detect(None, None)
                }
            }
        }
    }
}


config_enum!(CompilerConfig, CompilerKind);


#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub compiler: CompilerConfig,
    pub linker: LinkerConfig,
    #[default(vec![PathBuf::from("include")])]
    pub include_dirs: Vec<PathBuf>,
    #[default(Vec::new())]
    pub lib_dirs: Vec<PathBuf>,
    #[default(Vec::new())]
    pub libs: Vec<String>,
}


#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct DevProfile {
    #[default = "0"]
    pub opt_level: String,
    #[default = true]
    pub debug: bool,
    #[default = false]
    pub lto: bool,
    #[default = "regular"]
    pub lto_type: String,
    #[default = true]
    pub incremental: bool,
    #[default = 16]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = false]
    pub strip: bool,
}

#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct ReleaseProfile {
    #[default = "3"]
    pub opt_level: String,
    #[default = false]
    pub debug: bool,
    #[default = true]
    pub lto: bool,
    #[default = "fat"]
    pub lto_type: String,
    #[default = false]
    pub incremental: bool,
    #[default = 1]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = true]
    pub strip: bool,
}

#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct TestProfile {
    #[default = "0"]
    pub opt_level: String,
    #[default = true]
    pub debug: bool,
    #[default = false]
    pub lto: bool,
    #[default = "regular"]
    pub lto_type: String,
    #[default = true]
    pub incremental: bool,
    #[default = 16]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = false]
    pub strip: bool,
}

#[derive(Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct BenchProfile {
    #[default = "3"]
    pub opt_level: String,
    #[default = false]
    pub debug: bool,
    #[default = true]
    pub lto: bool,
    #[default = "fat"]
    pub lto_type: String,
    #[default = false]
    pub incremental: bool,
    #[default = 1]
    pub codegen_units: u32,
    #[default = "unwind"]
    pub panic: String,
    #[default = true]
    pub strip: bool,
}

#[derive(Deserialize, SmartDefault)]
pub struct Profiles {
    #[serde(default)]
    pub dev: DevProfile,
    #[serde(default)]
    pub release: ReleaseProfile,
    #[serde(default)]
    pub test: TestProfile,
    #[serde(default)]
    pub bench: BenchProfile,
}


#[derive(Clone)]
pub enum Profile {
    Dev(DevProfile),
    Release(ReleaseProfile),
    Test(TestProfile),
    Bench(BenchProfile),
}

impl CrowConfig {
    pub fn load() -> anyhow::Result<Self> {
        use anyhow::Context;
        let content = std::fs::read_to_string("crow.toml")
            .context("could not find `crow.toml` in current directory")?;
        let config: Self = toml::from_str(&content).context("failed to parse crow.toml")?;
        Ok(config)
    }
}

impl Profile {
    pub fn opt_level(&self) -> &str {
        match self {
            Profile::Dev(p) => &p.opt_level,
            Profile::Release(p) => &p.opt_level,
            Profile::Test(p) => &p.opt_level,
            Profile::Bench(p) => &p.opt_level,
        }
    }

    pub fn debug(&self) -> bool {
        match self {
            Profile::Dev(p) => p.debug,
            Profile::Release(p) => p.debug,
            Profile::Test(p) => p.debug,
            Profile::Bench(p) => p.debug,
        }
    }

    pub fn lto(&self) -> bool {
        match self {
            Profile::Dev(p) => p.lto,
            Profile::Release(p) => p.lto,
            Profile::Test(p) => p.lto,
            Profile::Bench(p) => p.lto,
        }
    }

    pub fn lto_type(&self) -> &str {
        match self {
            Profile::Dev(p) => &p.lto_type,
            Profile::Release(p) => &p.lto_type,
            Profile::Test(p) => &p.lto_type,
            Profile::Bench(p) => &p.lto_type,
        }
    }

    pub fn incremental(&self) -> bool {
        match self {
            Profile::Dev(p) => p.incremental,
            Profile::Release(p) => p.incremental,
            Profile::Test(p) => p.incremental,
            Profile::Bench(p) => p.incremental,
        }
    }

    pub fn codegen_units(&self) -> u32 {
        match self {
            Profile::Dev(p) => p.codegen_units,
            Profile::Release(p) => p.codegen_units,
            Profile::Test(p) => p.codegen_units,
            Profile::Bench(p) => p.codegen_units,
        }
    }

    pub fn panic(&self) -> &str {
        match self {
            Profile::Dev(p) => &p.panic,
            Profile::Release(p) => &p.panic,
            Profile::Test(p) => &p.panic,
            Profile::Bench(p) => &p.panic,
        }
    }

    pub fn strip(&self) -> bool {
        match self {
            Profile::Dev(p) => p.strip,
            Profile::Release(p) => p.strip,
            Profile::Test(p) => p.strip,
            Profile::Bench(p) => p.strip,
        }
    }

    pub fn apply_to_compile_flags(&self, flags: &mut crate::builder::flags::Flags) {
        match self.opt_level() {
            "0" => flags.no_optimization(),
            "1" => flags.optimization_level(1),
            "2" => flags.optimization_level(2),
            "3" => flags.optimization_level(3),
            "s" | "z" => flags.optimize_size(),
            _ => flags.optimization_level(2),
        };

        if self.debug() {
            flags.debug_info();
        } else {
            flags.no_debug_info();
        }

        if self.lto() {
            match flags.get_compiler_kind() {
                CompilerKind::Msvc => {
                    flags.link_time_optimization();
                }
                CompilerKind::Clang | CompilerKind::ClangPP => {
                    match self.lto_type() {
                        "thin" => flags.thin_lto(),
                        "fat" | "full" => flags.fat_lto(),
                        _ => flags.link_time_optimization(),
                    };
                }
                CompilerKind::Gcc | CompilerKind::Gpp => {
                    flags.link_time_optimization();
                }
            }
        } else {
            flags.no_link_time_optimization();
        }

        if flags.get_compiler_kind().is_msvc() {
            if self.opt_level() != "0" || !self.debug() {
                flags.define("NDEBUG", None::<String>);
                flags.multi_threaded_dll();
            } else {
                flags.define("_DEBUG", None::<String>);
                flags.multi_threaded_dll_debug();
            }

            if self.debug() {
                flags.debug_type("Zi".to_string());
            }
        }
    }

    pub fn apply_to_link_flags(&self, flags: &mut crate::builder::flags::Flags) {
        if self.lto() {
            match flags.get_compiler_kind() {
                CompilerKind::Msvc => {
                    flags.link_time_optimization();
                }
                CompilerKind::Clang | CompilerKind::ClangPP => {
                    match self.lto_type() {
                        "thin" => flags.thin_lto(),
                        "fat" | "full" => flags.fat_lto(),
                        _ => flags.link_time_optimization(),
                    };
                }
                CompilerKind::Gcc | CompilerKind::Gpp => {
                    flags.link_time_optimization();
                }
            }
        } else {
            flags.no_link_time_optimization();
        }

        if flags.get_compiler_kind().is_msvc() && self.debug() {
            flags.debug_info();
        }
    }
}