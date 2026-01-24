use crate::config::CrowConfig;
use anyhow::Context;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Project {
    pub config: CrowConfig,
    pub root: PathBuf,
    pub release: bool,
}

impl Project {
    pub fn new(config: CrowConfig, release: bool) -> anyhow::Result<Self> {
        let root = std::env::current_dir().context("failed to get current directory")?;

        Ok(Self {
            config,
            root,
            release,
        })
    }

    pub fn find_sources(&self) -> Vec<PathBuf> {
        WalkDir::new("src")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry.path().extension().map_or(false, |ext| {
                    matches!(ext.to_str(), Some("cpp" | "c" | "cc"))
                })
            })
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    pub fn target_dir(&self) -> PathBuf {
        self.root.join("target")
    }

    pub fn profile_dir(&self) -> PathBuf {
        self.target_dir()
            .join(if self.release { "release" } else { "debug" })
    }

    pub fn build_dir(&self) -> PathBuf {
        self.profile_dir()
    }

    pub fn output_path(&self) -> PathBuf {
        self.profile_dir().join(&self.output_name())
    }

    pub fn output_name(&self) -> String {
        if cfg!(windows) {
            format!("{}.exe", self.config.package.name)
        } else {
            self.config.package.name.clone()
        }
    }

    pub fn get_profile(&self) -> &crate::config::Profile {
        if self.release {
            &self.config.profile.release
        } else {
            &self.config.profile.dev
        }
    }

    pub fn create_dirs(&self) -> Result<(), std::io::Error> {
        let dirs = vec![self.target_dir(), self.profile_dir()];

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

    pub fn compiler_kind(&self) -> crate::builder::CompilerKind {
        crate::builder::CompilerKind::detect(&self.config.build.compiler)
    }
}
