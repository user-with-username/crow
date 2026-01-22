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
        let profile = self.project.get_profile();
        let flags = Flags::new(profile, &self.project.config.build, self.project.compiler_kind());
        let output = self.project.output_path();

        let mut cmd = Command::new(self.compiler_exe);
        cmd.args(flags.link_flags())
           .args(self.objects)
           .arg("-o")
           .arg(&output);

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
