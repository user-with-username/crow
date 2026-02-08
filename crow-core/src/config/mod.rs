use serde::Deserialize;
use std::path::PathBuf;

use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;

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

#[derive(Deserialize, Default, Clone)]
pub struct LinkerConfig {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub kind: Option<LinkerKind>,
}

#[derive(Deserialize, Default, Clone)]
pub struct CompilerConfig {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub kind: Option<CompilerKind>,
}

#[derive(Deserialize, Default, Clone)]
pub struct BuildConfig {
    #[serde(default)]
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub linker: LinkerConfig,
    #[serde(default)]
    pub include_dirs: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub lib_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub libs: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct Profiles {
    #[serde(default)]
    pub dev: Profile,
    #[serde(default)]
    pub release: Profile,
    #[serde(default)]
    pub test: Profile,
    #[serde(default)]
    pub bench: Profile,
}

#[derive(Deserialize, Default, Clone)]
pub struct Profile {
    #[serde(default = "default_opt_level")]
    pub opt_level: String,
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default)]
    pub lto: bool,
    #[serde(default)]
    pub lto_type: Option<String>,
    #[serde(default)]
    pub incremental: Option<bool>,
    #[serde(default)]
    pub codegen_units: Option<u32>,
    #[serde(default)]
    pub panic: Option<String>,
    #[serde(default)]
    pub strip: Option<bool>,
}

fn default_opt_level() -> String {
    "0".to_string()
}

fn default_debug() -> bool {
    true
}

fn default_include_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from("include")]
}

impl BuildConfig {
    pub fn get_include_dirs(&self) -> Vec<PathBuf> {
        self.include_dirs
            .clone()
            .unwrap_or_else(default_include_dirs)
    }
}

impl CrowConfig {
    pub fn load() -> anyhow::Result<Self> {
        use anyhow::Context;

        let content = std::fs::read_to_string("crow.toml")
            .context("could not find `crow.toml` in current directory")?;

        let config: Self = toml::from_str(&content).context("failed to parse crow.toml")?;

        Ok(config)
    }

    pub fn get_profile(&self, name: &str) -> &Profile {
        match name {
            "dev" => &self.profile.dev,
            "release" => &self.profile.release,
            "test" => &self.profile.test,
            "bench" => &self.profile.bench,
            _ => &self.profile.dev,
        }
    }
}

impl Profile {
    pub fn apply_to_compile_flags(&self, flags: &mut crate::builder::flags::Flags) {
        match self.opt_level.as_str() {
            "0" => {
                flags.no_optimization();
            }
            "1" => {
                flags.optimization_level(1);
            }
            "2" => {
                flags.optimization_level(2);
            }
            "3" => {
                flags.optimization_level(3);
            }
            "s" => {
                flags.optimize_size();
            }
            "z" => {
                flags.optimize_size();
            }
            _ => {
                flags.optimization_level(2);
            }
        }

        if self.debug {
            flags.debug_info();
        } else {
            flags.no_debug_info();
        }

        if self.lto {
            let lto_type = self.lto_type.as_deref().unwrap_or("regular");
            match lto_type {
                "thin" => {
                    flags.thin_lto();
                }
                "fat" | "full" => {
                    flags.fat_lto();
                }
                _ => {
                    flags.link_time_optimization();
                }
            }
        } else {
            flags.no_link_time_optimization();
        }

        if flags.get_compiler_kind().is_msvc() {
            if self.opt_level != "0" || !self.debug {
                flags.define("NDEBUG", None::<String>);
                flags.multi_threaded_dll();
            } else {
                flags.define("_DEBUG", None::<String>);
                flags.multi_threaded_dll_debug();
            }

            if self.debug {
                flags.debug_type("Zi".to_string());
            }
        }
    }

    pub fn apply_to_link_flags(&self, flags: &mut crate::builder::flags::Flags) {
        if self.lto {
            let lto_type = self.lto_type.as_deref().unwrap_or("regular");
            match lto_type {
                "thin" => {
                    flags.thin_lto();
                }
                "fat" | "full" => {
                    flags.fat_lto();
                }
                _ => {
                    flags.link_time_optimization();
                }
            }
        } else {
            flags.no_link_time_optimization();
        }

        if flags.get_compiler_kind().is_msvc() {
            if self.debug {
                flags.debug_info();
            }
        }
    }
}
