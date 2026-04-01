use crate::builder::paths::{
    DependencyFileNaming, DependencyFilePath, ObjectFilePath, SourceFilePath,
};
use crate::builder::{flags::CompilerFlags, incremental::CacheEntry, paths::ObjectFileNaming};
use anyhow::{Context, Result};
use serde::Serialize;
use std::hash::{DefaultHasher, Hash, Hasher};
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
    pub args: Vec<String>,
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
        let is_msvc = compiler_kind.is_msvc();

        let mut flags = CompilerFlags::new(compiler_kind);
        flags.compile_only();
        flags.standard_flags();
        profile.apply_to_compile_flags(&mut flags);

        if build_config.warnings_as_errors {
            flags.warnings_as_errors();
        }

        for inc in project.toolchain.system_include_dirs() {
            let inc_str = inc.to_string_lossy().to_string();
            flags.include_path(inc_str);
        }

        for inc in build_config.include_dirs.iter() {
            let inc_str = inc.to_string_lossy().to_string();
            flags.include_path(inc_str);
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
        let object_path_str = object_path.to_string_lossy().to_string();
        flags.object_output(&object_path_str);

        if compiler_kind.is_msvc() {
            let pdb_dir = obj_dir.join("pdb");
            std::fs::create_dir_all(&pdb_dir)?;
            let pdb_path = pdb_dir.join(format!("{}.pdb", source_file_stem));
            let pdb_path_str = pdb_path.to_string_lossy().to_string();
            flags.program_database(&pdb_path_str);
        }

        let built_flags = flags.build();

        let mut hasher = DefaultHasher::new();
        source.as_path().hash(&mut hasher);
        let hash = hasher.finish();
        let response_filename = format!("{}_{:x}.args", source_file_stem, hash);

        let source_path_str = source.as_path().to_string_lossy().to_string();

        let mut all_args_for_response = built_flags.clone();
        all_args_for_response.push(source_path_str.clone());

        let response_file_path = crow_utils::write_to(
            project.profile_dir(),
            &response_filename,
            &all_args_for_response,
            is_msvc,
        )?;

        let mut args_vec = built_flags;
        args_vec.push(source_path_str);

        let mut cmd = Command::new(compiler_exe);
        cmd.arg(format!("@{}", response_file_path.display()));

        Ok(Self {
            source: source.clone(),
            object: ObjectFilePath(object_path),
            dep_file,
            command: cmd,
            args: args_vec,
        })
    }

    pub fn to_compile_command(&self, directory: &Path) -> CompileCommand {
        let mut full_cmd = vec![self.command.get_program().to_string_lossy().to_string()];
        full_cmd.extend(self.args.clone());

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
