use crate::{flags::Flags, project::Project};
use anyhow::{Context, Result};
use std::process::Command;

pub struct LinkingBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
    objects: &'a [std::path::PathBuf],
    verbose: u8,
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(
        compiler_exe: &'a str,
        project: &'a Project,
        objects: &'a [std::path::PathBuf],
        verbose: u8,
    ) -> Self {
        Self {
            compiler_exe,
            project,
            objects,
            verbose,
        }
    }

    pub fn link(&self) -> Result<()> {
        let mut flags = Flags::new();
        
        flags.output_file(self.project.output_path().to_string_lossy().to_string());
        
        let config = &self.project.config;
        let build = &config.build;
        
        for raw_flag in &build.flags {
            flags.add_raw(raw_flag);
        }
        
        for lib in &build.libs {
            flags.link_library(lib);
        }

        for dir in &build.lib_dirs {
            flags.library_path(dir.to_string_lossy().to_string());
        }
        
        let args = flags.build();
        
        let mut cmd = Command::new(self.compiler_exe);
        cmd.args(&args);
        
        for obj in self.objects {
            cmd.arg(obj.to_string_lossy().to_string());
        }
        
        if self.verbose > 1 {
            crow_utils::status!("Command", "{:?}", cmd);
        }

        let status = cmd.status().context("linking failed")?;
        if !status.success() {
            anyhow::bail!("linking failed");
        }

        Ok(())
    }
}