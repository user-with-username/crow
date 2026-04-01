use crate::builder::flags::archiver_flags::ArchiverFlags;
use crate::builder::flags::LinkerFlags;
use crate::builder::paths::{ObjectFilePath, PdbFileNaming};
use crate::project::Project;
use anyhow::{Context, Result};
use crow_utils::show_output;
use std::process::Stdio;
use std::{fs, process::Command};

pub struct LinkingBuilder<'a> {
    linker_exe: &'a str,
    archiver_exe: &'a str,
    project: &'a Project,
    objects: &'a [ObjectFilePath],
}

impl<'a> LinkingBuilder<'a> {
    pub fn new(
        linker_exe: &'a str,
        archiver_exe: &'a str,
        project: &'a Project,
        objects: &'a [ObjectFilePath],
    ) -> Self {
        Self {
            linker_exe,
            archiver_exe,
            project,
            objects,
        }
    }

    pub fn link(&self) -> Result<()> {
        let p_type = &self.project.package.r#type;

        if p_type.is_bin() || p_type.is_shared() {
            self.link_dynamic()
        } else if p_type.is_static() {
            self.link_static()
        } else {
            Ok(())
        }
    }

    fn link_dynamic(&self) -> Result<()> {
        let mut flags = LinkerFlags::new(self.project.compiler_kind());
        let p_type = &self.project.package.r#type;
        let is_msvc = self.project.compiler_kind().is_msvc();

        flags.output_file(self.project.output_path().to_string_lossy().into_owned());

        if p_type.is_shared() {
            flags.shared_library();
        }

        flags.standard_flags();
        self.project.profile.apply_to_link_flags(&mut flags);

        self.setup_platform_artifacts(&mut flags)?;

        self.apply_library_flags(&mut flags);

        let mut args = flags.build();

        for obj in self.objects {
            args.push(obj.as_path().to_string_lossy().into_owned());
        }

        let response_filename = format!(
            "{}_{}.args",
            self.project.package.name,
            if p_type.is_shared() { "shared" } else { "bin" }
        );
        let response_file_path = crow_utils::write_to(
            self.project.profile_dir(),
            &response_filename,
            &args,
            is_msvc,
        )?;

        let mut cmd = Command::new(self.linker_exe);
        cmd.arg(format!("@{}", response_file_path.display()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        self.execute(cmd, "linking")
    }

    fn link_static(&self) -> Result<()> {
        let output_path = self.project.output_path();
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut flags = ArchiverFlags::new(self.project.archiver_kind());
        flags.output_file(output_path.to_string_lossy().into_owned());

        let mut args = flags.build();

        for obj in self.objects {
            args.push(obj.as_path().to_string_lossy().into_owned());
        }

        for raw in self.project.config.build.archiver.flags() {
            args.push(raw.clone());
        }

        let response_filename = format!("{}_static.args", self.project.package.name);
        let response_file_path = crow_utils::write_to(
            self.project.profile_dir(),
            &response_filename,
            &args,
            self.project.compiler_kind().is_msvc(),
        )?;

        let mut cmd = Command::new(self.archiver_exe);
        cmd.arg(format!("@{}", response_file_path.display()));

        self.execute(cmd, "archiving")
    }

    fn setup_platform_artifacts(&self, flags: &mut LinkerFlags) -> Result<()> {
        let kind = self.project.compiler_kind();
        let p_type = &self.project.package.r#type;
        let pkg_name = &self.project.package.name;

        if kind.is_msvc() {
            let pdb_dir = self.project.profile_dir().join("pdb");
            fs::create_dir_all(&pdb_dir)?;
            let pdb_path = PdbFileNaming::generate(&pdb_dir, pkg_name);
            flags.link_program_database(pdb_path.to_string_lossy().into_owned());
        }

        if p_type.generate_import_lib() {
            let output_name = p_type.output_name(pkg_name);

            let ext = if kind.is_msvc() { "lib" } else { "a" };
            let prefix = if kind.is_gnu() { "lib" } else { "" };

            let import_lib_name = format!("{}{}.{}", prefix, output_name, ext);
            let import_lib_path = self.project.profile_dir().join(import_lib_name);

            flags.import_library(import_lib_path.to_string_lossy().into_owned());
        }

        Ok(())
    }

    fn apply_library_flags(&self, flags: &mut LinkerFlags) {
        let build = &self.project.config.build;
        for dir in self.project.toolchain.system_library_dirs() {
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
    }

    fn execute(&self, mut cmd: Command, action: &str) -> Result<()> {
        let output = cmd
            .output()
            .with_context(|| format!("failed to execute {}", action))?;
        if !output.status.success() {
            show_output!(output, self.project);
            anyhow::bail!(
                "{} failed with exit code {}",
                action,
                output.status.code().unwrap_or(-1)
            );
        }
        Ok(())
    }
}
