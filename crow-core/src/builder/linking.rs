use crate::builder::paths::ObjectFilePath;
use crate::{builder::flags::Flags, project::Project};
use anyhow::{Context, Result};
use std::process::Command;

pub struct LinkingBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
    objects: &'a [ObjectFilePath],
    verbose: u8,
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(
        compiler_exe: &'a str,
        project: &'a Project,
        objects: &'a [ObjectFilePath],
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

        flags.output_file(self.project.output_path().to_string_lossy().into_owned());

        let config = &self.project.config;
        let build = &config.build;

        for raw_flag in &build.flags {
            flags.add_raw(raw_flag.clone());
        }

        for lib in &build.libs {
            flags.link_library(lib.clone());
        }

        for dir in &build.lib_dirs {
            flags.library_path(dir.to_string_lossy().into_owned());
        }

        let linker_args = flags.build();

        let mut cmd = Command::new(self.compiler_exe);
        cmd.args(&linker_args);

        for obj in self.objects {
            cmd.arg(obj.as_path());
        }

        if self.verbose > 1 {
            crow_utils::status!("Linking command", "{:?}", cmd);
        }

        let status = cmd.status().context("failed to execute linker")?;

        if !status.success() {
            anyhow::bail!(
                "linking failed with exit code {}",
                status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }
}
