#[cfg(test)]
mod tests {
    use crow_core::config::{BuildConfig, CrowConfig, Package, ProjectType};
    use crow_core::project::BuildSession;
    use crow_core::Project;
    use tempfile::tempdir;

    fn project_header_only() -> anyhow::Result<Project> {
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
        Project::new(config, dir.path().to_path_buf(), "dev", None)
    }

    #[test]
    fn build_session_new() {
        let s = BuildSession::new("dev", Some(4));
        assert_eq!(s.profile_name, "dev");
        assert_eq!(s.jobs, Some(4));
        assert_eq!(s.built_count, 0);
    }

    #[test]
    fn project_find_sources_empty_when_no_src_tree() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let config = CrowConfig {
            package: Some(Package::new("nop", "0.1.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        let project = Project::new(config, dir.path().to_path_buf(), "dev", None)?;
        let sources = project.find_sources();
        assert!(sources.is_empty());
        Ok(())
    }

    #[test]
    fn project_should_build_false_for_header_only() -> anyhow::Result<()> {
        let project = project_header_only()?;
        assert!(!project.should_build("lockhash")?);
        Ok(())
    }
}
