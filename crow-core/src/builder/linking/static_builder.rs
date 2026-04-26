use super::LinkingBuilder;
use crate::builder::flags::archiver_flags::ArchiverFlags;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub struct StaticLinkBuilder<'a> {
    builder: &'a LinkingBuilder<'a>,
}

impl<'a> StaticLinkBuilder<'a> {
    pub fn new(builder: &'a LinkingBuilder<'a>) -> Self {
        Self { builder }
    }

    pub fn build(&self) -> Result<()> {
        let project = self.builder.project();
        let output_path = project.output_path();
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut flags = ArchiverFlags::new(project.archiver_kind());
        flags.output_file(output_path.to_string_lossy().into_owned());

        let mut args = flags.build();

        for obj in self.builder.objects() {
            args.push(obj.as_path().to_string_lossy().into_owned());
        }

        for raw in project.config.build.archiver.flags() {
            args.push(raw.clone());
        }

        let response_filename = format!("{}_static.args", project.package.name);
        let response_file_path = crow_utils::write_to(
            project.profile_dir(),
            &response_filename,
            &args,
            project.compiler_kind().is_msvc(),
        )?;

        let mut cmd = Command::new(self.builder.archiver_exe());
        cmd.arg(format!("@{}", response_file_path.display()));

        self.builder.run_command(cmd, "archiving")
    }
}
