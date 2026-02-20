use crate::builder::flags::Flags;
use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::config::profile::{BenchProfile, DevProfile, ReleaseProfile, TestProfile};

#[derive(Clone)]
pub enum Profile {
    Dev(DevProfile),
    Release(ReleaseProfile),
    Test(TestProfile),
    Bench(BenchProfile),
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

    pub fn apply_to_compile_flags(&self, flags: &mut Flags) {
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

    pub fn apply_to_link_flags(&self, flags: &mut Flags) {
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
