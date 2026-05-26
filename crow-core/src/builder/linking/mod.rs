use crate::builder::flags::LinkerFlags;
use crate::builder::paths::ObjectFilePath;
use crate::project::Project;
use anyhowed::{Context, Result};
use crow_utils::progress::ProgressBar;
use crow_utils::show_output;
use std::process::Command;

mod dynamic_builder;
mod platform_builder;
mod static_builder;

pub use dynamic_builder::DynamicLinkBuilder;
pub use static_builder::StaticLinkBuilder;

pub struct LinkingBuilder<'a> {
    linker_exe: &'a str,
    archiver_exe: &'a str,
    project: &'a Project,
    objects: &'a [ObjectFilePath],
    progress: Option<&'a ProgressBar>,
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(
        linker_exe: &'a str,
        archiver_exe: &'a str,
        project: &'a Project,
        objects: &'a [ObjectFilePath],
        progress: Option<&'a ProgressBar>,
    ) -> Self {
        Self {
            linker_exe,
            archiver_exe,
            project,
            objects,
            progress,
        }
    }

    pub fn link(&self) -> Result<()> {
        let p_type = &self.project.package.r#type;

        if p_type.is_bin() || p_type.is_shared() {
            DynamicLinkBuilder::new(self).build()
        } else if p_type.is_static() {
            StaticLinkBuilder::new(self).build()
        } else {
            Ok(())
        }
    }

    pub fn link_executable(&self, output: &std::path::Path, extra_inputs: &[String]) -> Result<()> {
        DynamicLinkBuilder::new(self).build_executable(output, self.objects, extra_inputs, false)
    }

    fn run_command(&self, mut cmd: Command, action: &str) -> Result<()> {
        let output = cmd
            .output()
            .with_context(|| format!("failed to execute {}", action))?;

        if !output.status.success() {
            if let Some(pb) = self.progress {
                pb.finish();
            }
            show_output!(output, self.project);
            anyhowed::bail!(
                "{} failed with exit code {}",
                action,
                output.status.code().unwrap_or(-1)
            );
        }
        Ok(())
    }

    pub fn linker_exe(&self) -> &str {
        self.linker_exe
    }

    pub fn archiver_exe(&self) -> &str {
        self.archiver_exe
    }

    pub fn project(&self) -> &Project {
        self.project
    }

    pub fn objects(&self) -> &[ObjectFilePath] {
        self.objects
    }

    fn apply_library_flags(&self, flags: &mut LinkerFlags) {
        let build = &self.project.config.build;
        for dir in self
            .project
            .toolchain()
            .map(|t| t.system_library_dirs())
            .unwrap_or_default()
        {
            flags.library_path(dir.to_string_lossy().into_owned());
        }
        for dir in &build.lib_dirs {
            flags.library_path(dir.to_string_lossy().into_owned());
        }
        for lib in &build.libs {
            flags.link_library(lib.clone());
        }
        for raw in build.linker.flags() {
            flags.add_raw(raw);
        }
        if let Some(resolved_deps) = self.project.resolved_deps.as_ref() {
            for sys_lib in &resolved_deps.system_libs {
                flags.link_library(sys_lib.clone());
            }
        }
    }
}
