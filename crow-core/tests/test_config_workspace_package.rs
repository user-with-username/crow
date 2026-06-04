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
    fn target_cfg_section_applied() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let is_windows = cfg!(target_os = "windows");

        let toml_content = r#"
[package]
name = "test-target"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(windows)".build]
libs = ["advapi32"]
"#;

        if is_windows {
            fs::write(dir.path().join("crow.toml"), toml_content)?;
            let (cfg, _) = CrowConfig::load_from(dir.path(), false)?;
            let libs: Vec<&str> = cfg.build.libs.iter().map(|s| s.as_str()).collect();
            assert!(
                libs.contains(&"advapi32"),
                "windows target section should add advapi32, got: {:?}",
                libs
            );
        }
        Ok(())
    }

    #[test]
    fn target_cfg_section_unix() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let is_unix = cfg!(unix);

        let toml_content = r#"
[package]
name = "test-target"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(unix)".build]
preprocessor_defines = ["_GNU_SOURCE"]
"#;

        if is_unix {
            fs::write(dir.path().join("crow.toml"), toml_content)?;
            let (cfg, _) = CrowConfig::load_from(dir.path(), false)?;
            let defines: Vec<&str> = cfg
                .build
                .preprocessor_defines
                .iter()
                .map(|s| s.as_str())
                .collect();
            assert!(
                defines.contains(&"_GNU_SOURCE"),
                "unix target section should add _GNU_SOURCE, got: {:?}",
                defines
            );
        }
        Ok(())
    }

    #[test]
    fn target_cfg_section_unmatched_removed() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let is_windows = cfg!(target_os = "windows");

        // Write a section that does NOT match this platform
        let toml_content = if is_windows {
            r#"
[package]
name = "test-target"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(unix)".build]
libs = ["pthread"]
"#
        } else {
            r#"
[package]
name = "test-target"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(windows)".build]
libs = ["advapi32"]
"#
        };

        fs::write(dir.path().join("crow.toml"), toml_content)?;
        let (cfg, _) = CrowConfig::load_from(dir.path(), false)?;
        // The unmatched target section should NOT have been applied
        let libs: Vec<&str> = cfg.build.libs.iter().map(|s| s.as_str()).collect();
        assert!(
            !libs.contains(&"advapi32") && !libs.contains(&"pthread"),
            "unmatched target section should not be applied, got: {:?}",
            libs
        );
        Ok(())
    }
    #[test]
    fn conditional_dependency_with_target() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let is_windows = cfg!(target_os = "windows");

        let toml_content = r#"
[package]
name = "test-deps"
version = "0.1.0"

[dependencies]
always-dep = "1.0"
"#;

        fs::write(dir.path().join("crow.toml"), toml_content)?;

        // Also write a conditional dep via target section
        let toml_with_target = if is_windows {
            r#"
[package]
name = "test-deps"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(windows)".dependencies]
winhttp = "0.1"
"#
        } else {
            r#"
[package]
name = "test-deps"
version = "0.1.0"

[build]
src_dirs = ["src"]

[target."cfg(unix)".dependencies]
pthread = "0.1"
"#
        };

        fs::write(dir.path().join("crow.toml"), toml_with_target)?;
        let (cfg, _) = CrowConfig::load_from(dir.path(), false)?;

        if is_windows {
            assert!(cfg.dependencies.contains_key("winhttp"));
        } else {
            assert!(cfg.dependencies.contains_key("pthread"));
        }
        Ok(())
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
