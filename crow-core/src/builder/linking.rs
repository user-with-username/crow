use crate::builder::paths::ObjectFilePath;
use crate::{builder::flags::Flags, project::Project};
use anyhow::{Context, Result};
use std::{fs, process::Command};

pub struct LinkingBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
    objects: &'a [ObjectFilePath],
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(
        compiler_exe: &'a str,
        project: &'a Project,
        objects: &'a [ObjectFilePath],
    ) -> Self {
        Self {
            compiler_exe,
            project,
            objects,
        }
    }

    pub fn link(&self) -> Result<()> {
        let mut flags = Flags::new(self.project.compiler_kind());
        let config = &self.project.config;
        let build = &config.build;

        flags.no_logo();
        flags.output_file(self.project.output_path().to_string_lossy().into_owned());

        if self.project.compiler_kind().is_msvc() {
            let profile_dir = self.project.profile_dir();
            let pdb_dir = profile_dir.join("pdb");
            fs::create_dir_all(&pdb_dir)?;
            let pdb_path = pdb_dir.join(format!("{}.pdb", config.package.name));
            flags.link_program_database(pdb_path.to_string_lossy().into_owned());
        }

        for dir in &build.lib_dirs {
            flags.library_path(dir.to_string_lossy().into_owned());
        }

        for lib in &build.libs {
            flags.link_library(lib.clone());
        }

        if self.project.compiler_kind().is_msvc() {
            if self.project.release {
                flags.multi_threaded_dll();
            } else {
                flags.multi_threaded_dll_debug();
            }
        }

        for raw_flag in &build.flags {
            flags.add_raw(raw_flag);
        }

        let mut cmd = Command::new(self.compiler_exe);
        cmd.args(flags.build());

        for obj in self.objects {
            cmd.arg(obj.as_path());
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
