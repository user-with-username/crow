#[cfg(test)]
mod tests {
    use crow_core::config::{BuildConfig, CrowConfig, Package, ProjectType};
    use crow_core::project::BuildSession;
    use crow_core::Project;
    use tempfile::tempdir;

    fn project_header_only() -> anyhowed::Result<Project> {
        let dir = tempdir()?;
        let config = CrowConfig {
            package: Some(Package {
                name: "hdr".to_string(),
                version: "1.0.0".to_string(),
                r#type: ProjectType::HeaderOnly,
                ..Default::default()
            }),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        Project::new(config, dir.path().to_path_buf(), "dev", None, None)
    }

    #[test]
    fn build_session_new() {
        let s = BuildSession::new("dev", Some(4), false);
        assert_eq!(s.profile_name, "dev");
        assert_eq!(s.jobs, Some(4));
        assert_eq!(s.built_count, 0);
    }

    #[test]
    fn project_find_sources_empty_when_no_src_tree() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let config = CrowConfig {
            package: Some(Package::new("nop", "0.1.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        let project = Project::new(config, dir.path().to_path_buf(), "dev", None, None)?;
        let sources = project.find_sources();
        assert!(sources.is_empty());
        Ok(())
    }

    #[test]
    fn project_find_tests_uses_test_dirs() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        std::fs::create_dir_all(dir.path().join("tests"))?;
        std::fs::write(
            dir.path().join("tests/test_one.cpp"),
            "int main() { return 0; }",
        )?;
        std::fs::create_dir_all(dir.path().join("src"))?;
        std::fs::write(dir.path().join("src/main.cpp"), "int main() { return 0; }")?;

        let config = CrowConfig {
            package: Some(Package::new("nop", "0.1.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        let project = Project::new(config, dir.path().to_path_buf(), "dev", None, None)?;

        assert_eq!(project.find_sources().len(), 1);
        assert_eq!(project.find_tests().len(), 1);
        assert!(project.find_tests()[0].ends_with("test_one.cpp"));
        Ok(())
    }

    #[test]
    fn project_should_build_false_for_header_only() -> anyhowed::Result<()> {
        let project = project_header_only()?;
        assert!(!project.should_build("lockhash")?);
        Ok(())
    }
}
