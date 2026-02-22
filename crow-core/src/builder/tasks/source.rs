use crate::builder::paths::{
    DependencyFileNaming, DependencyFilePath, ObjectFilePath, SourceFilePath,
};
use crate::builder::{flags::CompilerFlags, incremental::CacheEntry, paths::ObjectFileNaming};
use anyhow::{Context, Result};
use crow_utils::normalize_path;
use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Serialize, Clone)]
pub struct CompileCommand {
    pub directory: String,
    pub command: String,
    pub file: String,
}

#[derive(Debug)]
pub struct SourceCompilationTask {
    pub source: SourceFilePath,
    pub object: ObjectFilePath,
    pub dep_file: DependencyFilePath,
    pub command: Command,
}

impl SourceCompilationTask {
    pub fn prepare(
        source: &SourceFilePath,
        compiler_exe: &str,
        build_config: &crate::config::BuildConfig,
        deps_dir: &Path,
        project: &crate::project::Project,
        profile: &crate::config::Profile,
    ) -> Result<Self> {
        let compiler_kind = project.compiler_kind();

        let mut flags = CompilerFlags::new(compiler_kind);
        flags.compile_only();
        flags.standard_flags();
        profile.apply_to_compile_flags(&mut flags);

        if build_config.warnings_as_errors {
            flags.warnings_as_errors();
        }

        for inc in project.toolchain.system_include_dirs() {
            flags.include_path(inc.to_string_lossy().into_owned());
        }

        for inc in build_config.include_dirs.iter() {
            flags.include_path(inc.to_string_lossy().into_owned());
        }

        for def in &build_config.preprocessor_defines {
            if let Some((name, value)) = def.split_once('=') {
                flags.define(name, Some(value));
            } else {
                flags.define(def.as_str(), None::<String>);
            }
        }

        for raw_flag in &build_config.compiler.flags().to_vec() {
            flags.add_raw(raw_flag.clone());
        }

        let source_file_stem = source
            .as_path()
            .file_stem()
            .context("Invalid file name")?
            .to_string_lossy();

        let dep_file = DependencyFileNaming::generate(deps_dir, source);
        flags.dependency_info(dep_file.as_path().to_string_lossy().into_owned());

        let obj_dir = if deps_dir.ends_with("deps") {
            deps_dir.parent().unwrap_or(deps_dir)
        } else {
            deps_dir
        };

        let object_path = ObjectFileNaming::generate(obj_dir, source, compiler_kind);
        flags.object_output(&normalize_path(&object_path.to_string_lossy()));

        if compiler_kind.is_msvc() {
            let pdb_dir = obj_dir.join("pdb");
            std::fs::create_dir_all(&pdb_dir)?;
            let pdb_path = pdb_dir.join(format!("{}.pdb", source_file_stem));
            flags.program_database(&normalize_path(&pdb_path.to_string_lossy()));
        }

        let built_flags = flags.build();

        let mut cmd = Command::new(compiler_exe);

        cmd.args(&built_flags);

        let source_path = normalize_path(&source.as_path().to_string_lossy());
        cmd.arg(&source_path);

        Ok(Self {
            source: source.clone(),
            object: ObjectFilePath(object_path),
            dep_file,
            command: cmd,
        })
    }

    pub fn to_compile_command(&self, directory: &Path) -> CompileCommand {
        let mut full_cmd = vec![self.command.get_program().to_string_lossy().to_string()];
        full_cmd.extend(
            self.command
                .get_args()
                .map(|a| a.to_string_lossy().to_string()),
        );

        CompileCommand {
            directory: directory.to_string_lossy().to_string(),
            command: full_cmd.join(" "),
            file: self.source.as_path().to_string_lossy().to_string(),
        }
    }

    pub fn to_cache_entry(&self, hash: String) -> CacheEntry {
        CacheEntry {
            hash,
            object: self.object.0.clone(),
        }
    }
}