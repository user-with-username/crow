#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crow_core::builder::flags::archiver_flags::ArchiverFlags;
    use crow_core::builder::kinds::archiver_kind::ArchiverKind;
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use crow_core::builder::kinds::linker_kind::LinkerKind;
    use crow_core::builder::linking::{LinkingBuilder, StaticLinkBuilder};
    use crow_core::builder::paths::ObjectFilePath;
    use crow_core::CrowConfig;
    use crow_core::Project;
    use std::fs;
    use tempfile::tempdir;

    /// Toolchain kinds that `detect_toolchain` can satisfy on this host.
    fn toolchain_kinds_for_platform() -> (CompilerKind, LinkerKind, ArchiverKind) {
        if cfg!(windows) {
            (CompilerKind::Msvc, LinkerKind::Link, ArchiverKind::Lib)
        } else if cfg!(target_os = "macos") {
            (CompilerKind::ClangPP, LinkerKind::Lld, ArchiverKind::LlvmAr)
        } else {
            (CompilerKind::Gcc, LinkerKind::Ld, ArchiverKind::Ar)
        }
    }

    fn create_test_project(
        dir: &tempfile::TempDir,
        name: &str,
        archiver_kind: ArchiverKind,
    ) -> Result<Project> {
        let (compiler_kind, linker_kind, _) = toolchain_kinds_for_platform();

        let mut build_config = crow_core::config::BuildConfig::default();
        build_config.archiver = crow_core::config::ArchiverConfig::Simple(archiver_kind);
        build_config.compiler = crow_core::config::CompilerConfig::Simple(compiler_kind);
        build_config.linker = crow_core::config::LinkerConfig::Simple(linker_kind);

        let config = CrowConfig {
            package: Some(crow_core::config::Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                r#type: crow_core::config::ProjectType::StaticLib(Default::default()),
                authors: None,
                description: None,
                license: None,
                repository: None,
                standard: None,
            }),
            build: build_config,
            workspace: None,
            dependencies: Default::default(),
            profile: Default::default(),
        };

        Project::new(config, dir.path().to_path_buf(), "dev", None)
    }

    #[test]
    fn test_static_link_builder_new() -> Result<()> {
        let temp_dir = tempdir()?;
        let project = create_test_project(&temp_dir, "test_lib", toolchain_kinds_for_platform().2)?;
        let object_files: &[ObjectFilePath] = &[];
        let linking_builder = LinkingBuilder::new("test_lib", "lib", &project, object_files, None);
        let _builder = StaticLinkBuilder::new(&linking_builder);

        Ok(())
    }

    #[test]
    fn test_archiver_flags_construction() {
        let flags_llvm = ArchiverFlags::new(ArchiverKind::LlvmAr);
        let args_llvm = flags_llvm.build();
        assert!(!args_llvm.is_empty());

        assert!(args_llvm.contains(&"rcs".to_string()));
    }

    #[test]
    fn test_output_file_flag_format() {
        let mut flags = ArchiverFlags::new(ArchiverKind::LlvmAr);
        flags.output_file("output.a".to_string());
        let args = flags.build();
        assert!(args.iter().any(|arg| arg.contains("output.a")));
    }

    #[test]
    fn test_archiver_kind_methods() {
        assert!(ArchiverKind::Lib.is_msvc());
        assert!(!ArchiverKind::Ar.is_msvc());
        assert!(!ArchiverKind::LlvmAr.is_msvc());
        assert!(!ArchiverKind::Unknown.is_msvc());

        assert!(!ArchiverKind::Lib.is_gnu());
        assert!(ArchiverKind::Ar.is_gnu());
        assert!(ArchiverKind::LlvmAr.is_gnu());
        assert!(!ArchiverKind::Unknown.is_gnu());
    }

    #[test]
    fn test_linking_builder_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let obj_file_path = temp_dir.path().join("test.o");
        fs::write(&obj_file_path, "content")?;

        let project = create_test_project(&temp_dir, "test", toolchain_kinds_for_platform().2)?;
        let object_file = ObjectFilePath(obj_file_path);
        let object_files = &[object_file];
        let linking_builder = LinkingBuilder::new("test", "a", &project, object_files, None);

        assert!(!linking_builder.objects().is_empty());
        assert_eq!(linking_builder.objects().len(), 1);

        Ok(())
    }

    #[test]
    fn test_linking_builder_with_multiple_objects() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut object_paths = Vec::new();
        for i in 1..=3 {
            let obj_file_path = temp_dir.path().join(format!("test{}.o", i));
            fs::write(&obj_file_path, format!("object {}", i))?;
            object_paths.push(ObjectFilePath(obj_file_path));
        }

        let project =
            create_test_project(&temp_dir, "multi_lib", toolchain_kinds_for_platform().2)?;
        let linking_builder = LinkingBuilder::new("multi_lib", "a", &project, &object_paths, None);

        assert_eq!(linking_builder.objects().len(), 3);

        Ok(())
    }

    #[test]
    fn test_linking_builder_empty_objects() -> Result<()> {
        let temp_dir = tempdir()?;
        let project =
            create_test_project(&temp_dir, "empty_lib", toolchain_kinds_for_platform().2)?;
        let object_files: &[ObjectFilePath] = &[];
        let linking_builder = LinkingBuilder::new("empty_lib", "a", &project, object_files, None);

        assert!(linking_builder.objects().is_empty());

        Ok(())
    }
}
