use crate::builder::toolchain::{self, Toolchain};
use crate::config::{CrowConfig, Package};
use anyhow::{Context, Result};
use crow_utils::enviroment::Enviroment;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Project {
    pub config: CrowConfig,
    pub package: Package,
    pub root: PathBuf,
    pub workspace_root: PathBuf,
    pub profile: crate::config::Profile,
    pub profile_name: String,
    pub toolchain: Box<dyn Toolchain>,
}

impl Project {
    pub fn new(loaded_config: CrowConfig, manifest_dir: PathBuf, profile_name: &str) -> Result<Self> {
        let package = loaded_config
            .package
            .clone()
            .context("Manifest must have a [package] section")?;

        let profile = match profile_name {
            "dev" => crate::config::Profile::Dev(loaded_config.profile.dev.clone()),
            "release" => crate::config::Profile::Release(loaded_config.profile.release.clone()),
            "test" => crate::config::Profile::Test(loaded_config.profile.test.clone()),
            "bench" => crate::config::Profile::Bench(loaded_config.profile.bench.clone()),
            _ => crate::config::Profile::Dev(loaded_config.profile.dev.clone()),
        };

        let toolchain = toolchain::detect_toolchain(
            loaded_config.build.compiler.path().cloned(),
            Some(loaded_config.build.compiler.kind().clone()),
            loaded_config.build.linker.path().cloned(),
            Some(loaded_config.build.linker.kind().clone()),
            loaded_config.build.archiver.path().cloned(),
            Some(loaded_config.build.archiver.kind().clone()),
        )?;

        Ok(Self {
            config: loaded_config,
            package,
            workspace_root: manifest_dir.clone(),
            root: manifest_dir,
            profile,
            profile_name: profile_name.to_string(),
            toolchain,
        })
    }

    pub fn find_sources(&self) -> Vec<PathBuf> {
        let extensions = &self.config.build.src_extensions;
        WalkDir::new(self.root.join("src"))
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
        let base_name = self.package.r#type.display_name(&self.package.name);
        let filename = if self.package.r#type.needs_prefix() {
            format!("lib{}", base_name)
        } else {
            base_name.to_string()
        };

        self.profile_dir()
            .join(filename)
            .with_extension(self.package.r#type.extension())
    }

    pub fn output_name(&self) -> String {
        let base_name = self.package.r#type.display_name(&self.package.name);
        let filename = if self.package.r#type.needs_prefix() {
            format!("lib{}", base_name)
        } else {
            base_name.to_string()
        };
        let extension = self.package.r#type.extension();
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

    pub fn archiver_path(&self) -> &str {
        self.toolchain.archiver_path()
    }

    pub fn archiver_kind(&self) -> crate::builder::kinds::archiver_kind::ArchiverKind {
        self.toolchain.archiver_kind()
    }

    pub fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.toolchain.system_include_dirs()
    }
}