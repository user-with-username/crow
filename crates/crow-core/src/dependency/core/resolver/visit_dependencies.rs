use crate::dependency::core::constraint::DependencyConstraint;
use crate::dependency::graph::DependencyGraph;
use crate::dependency::DependencyResolver;
use anyhowed::Result;
use petgraph::graph::NodeIndex;
use std::path::Path;

impl DependencyResolver {
    pub(crate) fn visit_dependencies(
        &mut self,
        owner_idx: NodeIndex,
        deps: &crate::config::Dependencies,
        dev_deps: &crate::config::DevDependencies,
        owner_root: &Path,
        graph: &mut DependencyGraph,
        profile_name: &str,
        owner_name: &str,
    ) -> Result<()> {
        fn process_dep(
            resolver: &mut DependencyResolver,
            owner_idx: NodeIndex,
            dep_name: &str,
            spec: &crate::config::DependencySpec,
            owner_root: &Path,
            graph: &mut DependencyGraph,
            profile_name: &str,
            owner_name: &str,
        ) -> Result<()> {
            if spec.is_system() {
                return Ok(());
            }

            let constraint = DependencyConstraint::from_dependency_spec(spec, owner_root);
            resolver.check_dependency_conflict(profile_name, dep_name, constraint, owner_name)?;

            let resolved_dep =
                resolver.resolve_dependency(dep_name, spec, owner_root, profile_name)?;

            let crate::dependency::ResolvedPackage {
                root: canonical_root,
                config,
                source,
                checksum,
                is_wheel,
                build_flags,
                features,
                name: _,
                ..
            } = resolved_dep;

            let sub_dependencies = config.dependencies.clone();
            let sub_dev_dependencies = config.dev_dependencies.clone();

            let dep_package_name = config
                .package
                .as_ref()
                .map(|pkg| pkg.name.as_str())
                .unwrap_or(dep_name)
                .to_string();

            let dep_idx = graph.add_node(
                canonical_root.clone(),
                config,
                source,
                checksum,
                is_wheel,
                build_flags,
                features,
            )?;

            graph.add_edge(owner_idx, dep_idx)?;

            if !graph.is_visiting(&canonical_root) {
                graph.mark_visiting(canonical_root.clone());

                resolver.visit_dependencies(
                    dep_idx,
                    &sub_dependencies,
                    &sub_dev_dependencies,
                    &canonical_root,
                    graph,
                    profile_name,
                    &dep_package_name,
                )?;

                graph.unmark_visiting(&canonical_root);
            }

            Ok(())
        }

        for (dep_name, spec) in deps.iter() {
            process_dep(
                self,
                owner_idx,
                dep_name,
                spec,
                owner_root,
                graph,
                profile_name,
                owner_name,
            )?;
        }

        if profile_name == "test" || profile_name == "bench" {
            for (dep_name, spec) in dev_deps.iter() {
                process_dep(
                    self,
                    owner_idx,
                    dep_name,
                    spec,
                    owner_root,
                    graph,
                    profile_name,
                    owner_name,
                )?;
            }
        }

        Ok(())
    }
}
