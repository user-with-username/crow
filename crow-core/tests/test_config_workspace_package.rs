#[cfg(test)]
mod tests {
    use crow_core::config::{
        parse_standard, CrowConfig, Package, Profile, Profiles, ProjectType, Workspace,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_standard_accepts_cpp_year() {
        assert_eq!(parse_standard(Some(" 17 ")), Some(17));
        assert_eq!(parse_standard(Some("bad")), None);
        assert_eq!(parse_standard(None), None);
    }

    #[test]
    fn crow_config_is_virtual_from_toml() -> anyhowed::Result<()> {
        let virtual_cfg: CrowConfig = toml::from_str(
            r#"
[workspace]
members = ["crates/a"]
"#,
        )?;
        assert!(virtual_cfg.is_virtual());

        let root_cfg: CrowConfig = toml::from_str(
            r#"
[package]
name = "root"
version = "0.1.0"
"#,
        )?;
        assert!(!root_cfg.is_virtual());
        Ok(())
    }

    #[test]
    fn crow_config_load_from_minimal_toml() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        fs::write(
            dir.path().join("crow.toml"),
            r#"
[package]
name = "from-toml"
version = "2.0.0"
"#,
        )?;
        let (cfg, root) = CrowConfig::load_from(dir.path(), false)?;
        assert_eq!(root, dir.path());
        let pkg = cfg.package.expect("package");
        assert_eq!(pkg.name, "from-toml");
        assert_eq!(pkg.version, "2.0.0");
        Ok(())
    }

    #[test]
    fn crow_config_find_in_tree_walks_up() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        fs::write(
            dir.path().join("crow.toml"),
            r#"
[package]
name = "walk"
version = "1.0.0"
"#,
        )?;
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested)?;
        let (_cfg, root) = CrowConfig::find_in_tree(&nested)?;
        assert_eq!(root, dir.path());
        Ok(())
    }

    #[test]
    fn workspace_from_non_virtual_single_member() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        fs::write(
            dir.path().join("crow.toml"),
            r#"
[package]
name = "solo"
version = "1.0.0"
"#,
        )?;
        let ws = Workspace::load_from(dir.path())?;
        assert_eq!(ws.members().count(), 1);
        Ok(())
    }

    #[test]
    fn package_output_stem_static_lib() {
        let pkg = Package {
            name: "foo".to_string(),
            version: "0.1.0".to_string(),
            r#type: ProjectType::StaticLib(Default::default()),
            ..Default::default()
        };
        assert!(pkg.output_stem().starts_with("lib"));
        assert!(pkg.output_name().contains('.'));
    }

    #[test]
    fn project_type_predicates() {
        assert!(ProjectType::Bin(Default::default()).is_bin());
        assert!(ProjectType::StaticLib(Default::default()).is_static());
        assert!(!ProjectType::HeaderOnly.is_bin());
        assert_eq!(ProjectType::HeaderOnly.type_str(), "header-only");
    }

    #[test]
    fn profiles_default_and_profile_enum() {
        let profiles = Profiles::default();
        let _ = Profile::Dev(profiles.dev.clone());
        let _ = Profile::Release(profiles.release.clone());
        let _ = Profile::Test(profiles.test.clone());
        let _ = Profile::Bench(profiles.bench.clone());
    }
}
