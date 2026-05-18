#[cfg(test)]
mod tests {
    use crow_core::config::{BuildConfig, CrowConfig, Dependencies, Package, Profiles};
    use crow_core::dependency::{
        apply_dependency_standard, create_wheel, format_lock_dependencies, merge_dependency_inputs,
        DependencyGraph, DependencyResolver, GitDependencyFetcher, LockfileBuilder,
        ResolvedDependencyBuild, ResolvedPackage, VersionReq, WheelArtifacts, WheelType,
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
            profile: Profiles::default(),
        }
    }

    #[test]
    fn version_req_parse_star_and_matches() -> anyhow::Result<()> {
        let any = VersionReq::parse("*")?;
        assert!(any.matches("1.2.3"));

        let pinned = VersionReq::parse("=2.1.0")?;
        assert!(pinned.matches("2.1.0"));
        assert!(!pinned.matches("2.1.1"));

        let from_str: VersionReq = "1.0.0".parse()?;
        assert!(from_str.matches("1.0.0"));
        assert_eq!(from_str.as_str(), "^1.0.0");
        Ok(())
    }

    #[test]
    fn dependency_graph_topological_order() -> anyhow::Result<()> {
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
        )?;
        let idx_dep = g.add_node(
            root_b.clone(),
            empty_crow_config("leaf"),
            None,
            None,
            false,
            vec![],
        )?;
        g.add_edge(idx_root, idx_dep)?;
        let order = g.resolve_order()?;
        assert_eq!(order.len(), 2);
        Ok(())
    }

    #[test]
    fn dependency_graph_cycle_errors() -> anyhow::Result<()> {
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
        )?;
        let idx_b = g.add_node(
            root_b.clone(),
            empty_crow_config("pkg-b"),
            None,
            None,
            false,
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
            profile: Profiles::default(),
        };
        let lines = format_lock_dependencies(&cfg.dependencies, &by_name);
        assert!(lines.is_empty());
    }

    #[test]
    fn lockfile_builder_root_only() -> anyhow::Result<()> {
        let config = CrowConfig {
            package: Some(Package::new("root", "3.0.0")),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Dependencies::default(),
            profile: Profiles::default(),
        };
        let resolved = ResolvedDependencyBuild::default();
        let lock = LockfileBuilder::build(&config, &resolved)?;
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "root");
        Ok(())
    }

    #[test]
    fn create_wheel_unknown_directory() -> anyhow::Result<()> {
        let dir = tempdir()?;
        assert!(create_wheel(dir.path()).is_none());
        Ok(())
    }

    #[test]
    fn create_wheel_detects_cmake_layout() -> anyhow::Result<()> {
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
    fn create_wheel_detects_meson_layout() -> anyhow::Result<()> {
        let dir = tempdir()?;
        std::fs::write(dir.path().join("meson.build"), "project('x', 'c')\n")?;
        let wheel = create_wheel(dir.path()).expect("meson");
        assert!(matches!(wheel, WheelType::Meson(_)));
        Ok(())
    }

    #[test]
    fn create_wheel_detects_bazel_layout() -> anyhow::Result<()> {
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
}
