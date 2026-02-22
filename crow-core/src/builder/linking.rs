use crate::builder::paths::{ObjectFilePath, PdbFileNaming};
use crate::{builder::flags::LinkerFlags, project::Project};
use anyhow::{Context, Result};
use crow_utils::show_output;
use std::process::Stdio;
use std::{fs, process::Command};

pub struct LinkingBuilder<'a> {
    linker_exe: &'a str,
    project: &'a Project,
    objects: &'a [ObjectFilePath],
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(linker_exe: &'a str, project: &'a Project, objects: &'a [ObjectFilePath]) -> Self {
        Self {
            linker_exe,
            project,
            objects,
        }
    }

    pub fn link(&self) -> Result<()> {
        let mut flags = LinkerFlags::new(self.project.compiler_kind());
        let config = &self.project.config;
        let build = &config.build;

        flags.output_file(self.project.output_path().to_string_lossy().into_owned());

        flags.standard_flags();
        self.project.profile.apply_to_link_flags(&mut flags);

        if self.project.compiler_kind().is_msvc() {
            let profile_dir = self.project.profile_dir();
            let pdb_dir = profile_dir.join("pdb");
            fs::create_dir_all(&pdb_dir)?;
            let pdb_path = PdbFileNaming::generate(&pdb_dir, &config.package.name);
            flags.link_program_database(pdb_path.to_string_lossy().into_owned());
        }

        for dir in self.project.toolchain.system_library_dirs() {
            flags.library_path(dir.to_string_lossy().into_owned());
        }

        for dir in &build.lib_dirs {
            flags.library_path(dir.to_string_lossy().into_owned());
        }

        for lib in &build.libs {
            flags.link_library(lib.clone());
        }

        for raw_flag in &build.linker.flags().to_vec() {
            flags.add_raw(raw_flag);
        }

        let mut cmd = Command::new(self.linker_exe);
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(flags.build());

        for obj in self.objects {
            cmd.arg(obj.as_path());
        }

        let output = cmd.output().context("failed to execute linker")?;

        if !output.status.success() {
            show_output!(output, self.project);
            anyhow::bail!(
                "linking failed with exit code {}",
                output.status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }
}