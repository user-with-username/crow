use super::LinkingBuilder;
use crate::builder::flags::LinkerFlags;
use crate::builder::linking::platform_builder::PlatformSetup;
use crate::builder::paths::ObjectFilePath;
use anyhowed::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// Whether a raw link argument is a library (as opposed to a flag or an object).
///
/// Static libraries of workspace members are passed by path, and on MinGW they
/// are named `lib<name>.lib`, so `.lib` must be recognised here as well.
/// Otherwise they end up before the object files, where GNU `ld` drops them.
fn is_library_link_arg(arg: &str) -> bool {
    const LIBRARY_EXTENSIONS: [&str; 5] = [".a", ".lib", ".so", ".dylib", ".dll"];

    arg.starts_with("-l")
        || arg.starts_with("-L")
        || LIBRARY_EXTENSIONS.iter().any(|ext| arg.ends_with(ext))
}

/// Ok, gcc and clang require object files before `-l` flags when linking static libraries
/// So i had to write this wheelchair function to order raw flags
/// But ye, i just needed a fast way to fix https://github.com/user-with-username/crow/issues/4#issue-4468736358
pub(crate) fn order_gcc_like_link_args(
    mut flags: Vec<String>,
    objects: &[ObjectFilePath],
) -> Vec<String> {
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

    // some cringe goes here
    // just because this will run only with gcc linkers i can push flags as strings. not via LinkerFlags
    // soooooo, why not
    if !libraries.is_empty() {
        args.push("-Wl,--start-group".to_string());
        args.extend(libraries);
        args.push("-Wl,--end-group".to_string());
    }

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
        let p_type = &project.package.r#type;
        let extra_inputs = Vec::new();
        if p_type.is_shared() {
            self.build_executable(
                &project.output_path(),
                self.builder.objects(),
                &extra_inputs,
                true,
            )
        } else {
            self.build_executable(
                &project.output_path(),
                self.builder.objects(),
                &extra_inputs,
                false,
            )
        }
    }

    pub fn build_executable(
        &self,
        output: &Path,
        objects: &[ObjectFilePath],
        extra_inputs: &[String],
        shared: bool,
    ) -> Result<()> {
        let project = self.builder.project();
        let mut flags = LinkerFlags::new(project.compiler_kind());
        let is_msvc = project.compiler_kind().is_msvc();

        flags.output_file(output.to_string_lossy().into_owned());

        if shared {
            flags.shared_library();
        }

        flags.standard_flags();
        project.profile.apply_to_link_flags(&mut flags);

        PlatformSetup::new(self.builder).configure(&mut flags)?;

        self.builder.apply_library_flags(&mut flags);

        for input in extra_inputs {
            flags.add_raw(input.clone());
        }

        let args = if is_msvc {
            let mut args = flags.build();
            for obj in objects {
                args.push(obj.as_path().to_string_lossy().into_owned());
            }
            args
        } else {
            order_gcc_like_link_args(flags.build(), objects)
        };

        let response_filename = format!(
            "link_{}_{}.args",
            output
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "out".into()),
            project.package.name
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