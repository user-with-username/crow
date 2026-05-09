#[cfg(test)]
mod tests {
    use crow_core::builder::CompilationBuilder;
    use crow_core::config::{BuildConfig, CrowConfig, Package};
    use crow_core::Project;
    use tempfile::tempdir;

    fn minimal_project() -> (tempfile::TempDir, Project) {
        let root = tempdir().unwrap();
        let config = CrowConfig {
            package: Some(Package {
                name: "compile_smoke".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            }),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        let project = Project::new(config, root.path().to_path_buf(), "dev", None)
            .expect("create project");
        (root, project)
    }

    #[test]
    fn compilation_builder_new() {
        let (_tmp, project) = minimal_project();
        let exe = project.compiler_path();
        let _builder = CompilationBuilder::new(exe, &project);
    }

    #[test]
    fn compile_with_no_source_files_yields_empty_objects() {
        let (_tmp, project) = minimal_project();
        let exe = project.compiler_path();
        let builder = CompilationBuilder::new(exe, &project);
        let objects = builder.compile(None).expect("compile with empty source set");
        assert!(objects.is_empty());
    }
}
