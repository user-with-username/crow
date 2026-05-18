use super::LinkingBuilder;
use crate::builder::flags::LinkerFlags;
use crate::builder::linking::platform_builder::PlatformSetup;
use crate::builder::paths::ObjectFilePath;
use anyhow::Result;
use std::process::{Command, Stdio};

fn is_library_link_arg(arg: &str) -> bool {
    arg.starts_with("-l")
        || arg.starts_with("-L")
        || arg.ends_with(".a")
        || arg.ends_with(".so")
        || arg.ends_with(".dylib")
}

/// GCC/Clang require object files before `-l` flags when linking static libraries.
fn order_gcc_like_link_args(mut flags: Vec<String>, objects: &[ObjectFilePath]) -> Vec<String> {
    let mut prefix = Vec::new();
    let mut libraries = Vec::new();

    for flag in flags.drain(..) {
        if is_library_link_arg(&flag) {
            libraries.push(flag);
        } else {
            prefix.push(flag);
        }
    }

    let mut args = prefix;
    for obj in objects {
        args.push(obj.as_path().to_string_lossy().into_owned());
    }
    args.extend(libraries);
    args
}

pub struct DynamicLinkBuilder<'a> {
    builder: &'a LinkingBuilder<'a>,
}

impl<'a> DynamicLinkBuilder<'a> {
    pub fn new(builder: &'a LinkingBuilder<'a>) -> Self {
        Self { builder }
    }

    pub fn build(&self) -> Result<()> {
        let project = self.builder.project();
        let mut flags = LinkerFlags::new(project.compiler_kind());
        let p_type = &project.package.r#type;
        let is_msvc = project.compiler_kind().is_msvc();

        flags.output_file(project.output_path().to_string_lossy().into_owned());

        if p_type.is_shared() {
            flags.shared_library();
        }

        flags.standard_flags();
        project.profile.apply_to_link_flags(&mut flags);

        PlatformSetup::new(self.builder).configure(&mut flags)?;

        self.builder.apply_library_flags(&mut flags);

        let args = if is_msvc {
            let mut args = flags.build();
            for obj in self.builder.objects() {
                args.push(obj.as_path().to_string_lossy().into_owned());
            }
            args
        } else {
            order_gcc_like_link_args(flags.build(), self.builder.objects())
        };

        let response_filename = format!(
            "{}_{}.args",
            project.package.name,
            if p_type.is_shared() { "shared" } else { "bin" }
        );
        let response_file_path =
            crow_utils::write_to(project.profile_dir(), &response_filename, &args, is_msvc)?;

        let mut cmd = Command::new(self.builder.linker_exe());
        cmd.arg(format!("@{}", response_file_path.display()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        self.builder.run_command(cmd, "linking")
    }
}
