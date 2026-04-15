use super::LinkingBuilder;
use crate::builder::flags::LinkerFlags;
use crate::builder::paths::PdbFileNaming;
use anyhow::Result;
use std::fs;

pub struct PlatformSetup<'a> {
    builder: &'a LinkingBuilder<'a>,
}

impl<'a> PlatformSetup<'a> {
    pub fn new(builder: &'a LinkingBuilder<'a>) -> Self {
        Self { builder }
    }

    pub fn configure(&self, flags: &mut LinkerFlags) -> Result<()> {
        let project = self.builder.project();
        let kind = project.compiler_kind();
        let p_type = &project.package.r#type;
        let pkg_name = &project.package.name;

        if kind.is_msvc() {
            let pdb_dir = project.profile_dir().join("pdb");
            fs::create_dir_all(&pdb_dir)?;
            let pdb_path = PdbFileNaming::generate(&pdb_dir, pkg_name);
            flags.link_program_database(pdb_path.to_string_lossy().into_owned());
        }

        if p_type.generate_import_lib() {
            let output_name = p_type.output_name(pkg_name);

            let ext = if kind.is_msvc() { "lib" } else { "a" };
            let prefix = if kind.is_gnu() { "lib" } else { "" };

            let import_lib_name = format!("{}{}.{}", prefix, output_name, ext);
            let import_lib_path = project.profile_dir().join(import_lib_name);

            flags.import_library(import_lib_path.to_string_lossy().into_owned());
        }

        Ok(())
    }
}