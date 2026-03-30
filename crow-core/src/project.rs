use crate::builder::toolchain::{self, Toolchain};
use crate::config::CrowConfig;
use anyhow::Context;
use crow_utils::enviroment::Enviroment;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Project {
    pub config: CrowConfig,
    pub root: PathBuf,
    pub profile: crate::config::Profile,
    pub profile_name: String,
    pub toolchain: Box<dyn Toolchain>,
}

impl Project {
    pub fn new(config: CrowConfig, profile_name: &str) -> anyhow::Result<Self> {
        let root = std::env::current_dir().context("failed to get current directory")?;
        let profile = match profile_name {
            "dev" => crate::config::Profile::Dev(config.profile.dev.clone()),
            "release" => crate::config::Profile::Release(config.profile.release.clone()),
            "test" => crate::config::Profile::Test(config.profile.test.clone()),
            "bench" => crate::config::Profile::Bench(config.profile.bench.clone()),
            _ => crate::config::Profile::Dev(config.profile.dev.clone()),
        };

        let toolchain = toolchain::detect_toolchain(
            config.build.compiler.path().cloned(),
            Some(config.build.compiler.kind().clone()),
            config.build.linker.path().cloned(),
            Some(config.build.linker.kind().clone()),
        )?;

        Ok(Self {
            config,
            root,
            profile,
            profile_name: profile_name.to_string(),
            toolchain,
        })
    }

    pub fn find_sources(&self) -> Vec<PathBuf> {
        let extensions = &self.config.build.src_extensions;
        WalkDir::new("src")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| extensions.iter().any(|e| e.as_str() == ext))
            })
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    pub fn target_dir(&self) -> PathBuf {
        Enviroment::target_dir()
    }

    pub fn profile_dir(&self) -> PathBuf {
        self.target_dir().join(&self.profile_name)
    }

    pub fn build_dir(&self) -> PathBuf {
        self.profile_dir()
    }

    pub fn output_path(&self) -> PathBuf {
        let base_name = self.config.r#type.display_name(&self.config.package.name);
        let filename = if self.config.r#type.needs_prefix() {
            format!("lib{}", base_name)
        } else {
            base_name.to_string()
        };
        self.profile_dir()
            .join(filename)
            .with_extension(self.config.r#type.extension())
    }

    pub fn output_name(&self) -> String {
        let base_name = self.config.r#type.display_name(&self.config.package.name);
        let filename = if self.config.r#type.needs_prefix() {
            format!("lib{}", base_name)
        } else {
            base_name.to_string()
        };
        let extension = self.config.r#type.extension();
        if extension.is_empty() {
            filename
        } else {
            format!("{}.{}", filename, extension)
        }
    }

    pub fn create_dirs(&self) -> Result<(), std::io::Error> {
        let dirs = vec![
            self.target_dir(),
            self.profile_dir(),
            self.profile_dir().join("deps"),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }

    pub fn clean(&self) -> Result<(), std::io::Error> {
        let target_dir = self.target_dir();
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        Ok(())
    }

    pub fn compiler_kind(&self) -> crate::builder::kinds::compiler_kind::CompilerKind {
        self.toolchain.compiler_kind()
    }

    pub fn linker_kind(&self) -> crate::builder::kinds::linker_kind::LinkerKind {
        self.toolchain.linker_kind()
    }

    pub fn compiler_path(&self) -> &str {
        self.toolchain.compiler_path()
    }

    pub fn linker_path(&self) -> &str {
        if let Some(path) = self.config.build.linker.path() {
            return path;
        }
        if self.compiler_kind().is_msvc() {
            self.toolchain.linker_path()
        } else {
            self.toolchain.compiler_path()
        }
    }

    pub fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.toolchain.system_include_dirs()
    }
}
