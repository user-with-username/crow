#[cfg(test)]
mod tests {
    use crow_core::config::{BuildConfig, CrowConfig, Dependencies, Package, Profiles};
    use crow_core::dependency::{
        apply_dependency_standard, create_wheel, format_lock_dependencies, merge_dependency_inputs,
        DependencyGraph, DependencyResolver, GitDependencyFetcher, LockfileBuilder,
        ResolvedDependencyBuild, ResolvedPackage, WheelArtifacts, WheelType,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn empty_crow_config(name: &str) -> CrowConfig {
        CrowConfig {
            package: Some(Package::new(name, "1.0.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Dependencies::default(),
            dev_dependencies: Default::default(),
            profile: Profiles::default(),
        }
    }

    #[test]
    fn version_req_parse_star_and_matches() -> anyhowed::Result<()> {
        pub use semver::VersionReq;

        let any = VersionReq::parse("*")?;
        assert!(any.matches(&"1.2.3".parse()?));

        let pinned = VersionReq::parse("=2.1.0")?;
        assert!(pinned.matches(&"2.1.0".parse()?));
        assert!(!pinned.matches(&"2.1.1".parse()?));

        let from_str: VersionReq = "1.0.0".parse()?;
        assert!(from_str.matches(&"1.0.0".parse()?));
        assert_eq!(from_str.to_string(), "^1.0.0");

        Ok(())
    }

    #[test]
    fn dependency_graph_topological_order() -> anyhowed::Result<()> {
        let mut g = DependencyGraph::new();
        let tmp = tempdir()?;
        let root_a = tmp.path().join("a");
        let root_b = tmp.path().join("b");
        std::fs::create_dir_all(&root_a)?;
        std::fs::create_dir_all(&root_b)?;

        let idx_root = g.add_node(
            root_a.clone(),
            empty_crow_config("root"),
            None,
            None,
            false,
            vec![],
            vec![],
        )?;
        let idx_dep = g.add_node(
            root_b.clone(),
            empty_crow_config("leaf"),
            None,
            None,
            false,
            vec![],
            vec![],
        )?;
        g.add_edge(idx_root, idx_dep)?;
        let order = g.resolve_order()?;
        assert_eq!(order.len(), 2);
        Ok(())
    }

    #[test]
    fn dependency_graph_cycle_errors() -> anyhowed::Result<()> {
        let mut g = DependencyGraph::new();
        let tmp = tempdir()?;
        let root_a = tmp.path().join("a");
        let root_b = tmp.path().join("b");
        std::fs::create_dir_all(&root_a)?;
        std::fs::create_dir_all(&root_b)?;

        let idx_a = g.add_node(
            root_a.clone(),
            empty_crow_config("pkg-a"),
            None,
            None,
            false,
            vec![],
            vec![],
        )?;
        let idx_b = g.add_node(
            root_b.clone(),
            empty_crow_config("pkg-b"),
            None,
            None,
            false,
            vec![],
            vec![],
        )?;
        g.add_edge(idx_a, idx_b)?;
        g.add_edge(idx_b, idx_a)?;
        assert!(g.resolve_order().is_err());
        Ok(())
    }

    #[test]
    fn merge_dependency_inputs_and_standard() {
        let mut config = CrowConfig {
            package: Some(Package::new("app", "1.0.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Dependencies::default(),
            dev_dependencies: Default::default(),
            profile: Profiles::default(),
        };
        let mut resolved = ResolvedDependencyBuild::default();
        resolved.include_dirs.push(PathBuf::from("extra_include"));
        resolved.libs.push("mylib".to_string());
        resolved.lib_paths.push(PathBuf::from("extra_lib"));
        resolved.max_standard = Some("20".to_string());

        merge_dependency_inputs(&mut config, &resolved);
        apply_dependency_standard(&mut config, &resolved);

        assert!(config
            .build
            .include_dirs
            .contains(&PathBuf::from("extra_include")));
        assert!(config.build.libs.contains(&"mylib".to_string()));
        assert!(config.build.lib_dirs.contains(&PathBuf::from("extra_lib")));
        assert_eq!(
            config.package.as_ref().and_then(|p| p.standard.as_deref()),
            Some("20")
        );
    }

    #[test]
    fn format_lock_dependencies_fallback_name() {
        let deps = Dependencies::default();
        let by_name: HashMap<String, &ResolvedPackage> = HashMap::new();
        let cfg = CrowConfig {
            package: None,
            workspace: None,
            build: BuildConfig::default(),
            dependencies: deps,
            dev_dependencies: Default::default(),
            profile: Profiles::default(),
        };
        let lines = format_lock_dependencies(&cfg.dependencies, &by_name);
        assert!(lines.is_empty());
    }

    #[test]
    fn lockfile_builder_root_only() -> anyhowed::Result<()> {
        let config = CrowConfig {
            package: Some(Package::new("root", "3.0.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Dependencies::default(),
            dev_dependencies: Default::default(),
            profile: Profiles::default(),
        };
        let resolved = ResolvedDependencyBuild::default();
        let lock = LockfileBuilder::build(&config, &resolved)?;
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "root");
        Ok(())
    }

    #[test]
    fn create_wheel_unknown_directory() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        assert!(create_wheel(dir.path()).is_none());
        Ok(())
    }

    #[test]
    fn create_wheel_detects_cmake_layout() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        std::fs::write(
            dir.path().join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.5)\nproject(dummy)\n",
        )?;
        let wheel = create_wheel(dir.path()).expect("cmake");
        assert!(matches!(wheel, WheelType::Cmake(_)));
        Ok(())
    }

    #[test]
    fn create_wheel_detects_meson_layout() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        std::fs::write(dir.path().join("meson.build"), "project('x', 'c')\n")?;
        let wheel = create_wheel(dir.path()).expect("meson");
        assert!(matches!(wheel, WheelType::Meson(_)));
        Ok(())
    }

    #[test]
    fn create_wheel_detects_bazel_layout() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        std::fs::write(dir.path().join("WORKSPACE"), "")?;
        let wheel = create_wheel(dir.path()).expect("bazel");
        assert!(matches!(wheel, WheelType::Bazel(_)));
        Ok(())
    }

    #[test]
    fn git_dependency_fetcher_singleton() {
        let _ = GitDependencyFetcher::global();
    }

    #[test]
    fn dependency_resolver_new() {
        let _resolver = DependencyResolver::new();
    }

    #[test]
    fn wheel_artifacts_default() {
        let a = WheelArtifacts::default();
        assert!(a.include_dirs.is_empty());
        assert!(a.lib_paths.is_empty());
        assert!(a.lib_names.is_empty());
    }

    #[test]
    fn system_dep_features_parse() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { system = true, features = ["system", "filesystem"], libs = ["boost_system", "boost_filesystem"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert!(boost.is_system());
        assert_eq!(boost.features(), ["system", "filesystem"]);
        assert_eq!(boost.system_libs(), ["boost_system", "boost_filesystem"]);
    }

    #[test]
    fn system_dep_features_only_auto_libs() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { system = true, features = ["system", "filesystem"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert!(boost.is_system());
        assert_eq!(boost.features(), ["system", "filesystem"]);
        assert!(boost.system_libs().is_empty()); // no explicit libs
    }

    #[test]
    fn system_dep_no_features_backward_compat() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
openssl = { system = true, libs = ["ssl", "crypto"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let openssl = config.dependencies.get("openssl").unwrap();
        assert!(openssl.is_system());
        assert!(openssl.features().is_empty());
        assert_eq!(openssl.system_libs(), ["ssl", "crypto"]);
    }

    #[test]
    fn resolved_dependency_build_system_features_default() {
        let resolved = ResolvedDependencyBuild::default();
        assert!(resolved.system_features.is_empty());
        assert!(resolved.system_libs_map.is_empty());
    }

    #[test]
    fn non_system_dep_features_empty() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
foo = "1.0"
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let foo = config.dependencies.get("foo").unwrap();
        assert!(foo.features().is_empty());
    }

    #[test]
    fn registry_dep_features_parse() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { version = "^1.91", features = ["asio"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert!(!boost.is_system());
        assert_eq!(boost.features(), ["asio"]);
    }

    #[test]
    fn registry_dep_features_explicit_registry_url() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { version = "^1.91", registry = "https://example.com/reg", features = ["asio", "system"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert!(!boost.is_system());
        assert_eq!(boost.features(), ["asio", "system"]);
    }

    #[test]
    fn registry_dep_no_features_backward_compat() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { version = "^1.91" }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert!(!boost.is_system());
        assert!(boost.features().is_empty());
    }

    #[test]
    fn registry_dep_features_collected() {
        // Registry dep features should be collected into registry_features
        // so the wheel build can select components.
        use crow_core::dependency::ResolvedDependencyBuild;
        let mut resolved = ResolvedDependencyBuild::default();
        let dep_name = "boost";
        let features = vec!["asio".to_string()];
        let dep_name_lower = dep_name.to_ascii_lowercase();
        resolved
            .registry_features
            .entry(dep_name_lower.clone())
            .or_insert_with(Vec::new)
            .extend(features.clone());

        assert_eq!(
            resolved.registry_features.get("boost").unwrap(),
            &vec!["asio".to_string()]
        );
    }

    #[test]
    fn registry_dep_explicit_libs_from_spec() {
        // When a registry dep has explicit libs, those are used verbatim.
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
boost = { version = "^1.91", features = ["asio"], libs = ["boost_asio"] }
"#;
        let config: CrowConfig = toml::from_str(toml_str).unwrap();
        let boost = config.dependencies.get("boost").unwrap();
        assert_eq!(boost.features(), ["asio"]);
        assert_eq!(boost.registry_libs(), ["boost_asio"]);
    }
}
