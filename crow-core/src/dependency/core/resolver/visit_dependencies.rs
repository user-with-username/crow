use crate::dependency::core::constraint::DependencyConstraint;
use crate::dependency::graph::DependencyGraph;
use crate::dependency::DependencyResolver;
use anyhow::Result;
use petgraph::graph::NodeIndex;
use std::path::Path;

impl DependencyResolver {
    pub(crate) fn visit_dependencies(
        &mut self,
        owner_idx: NodeIndex,
        deps: &crate::config::Dependencies,
        owner_root: &Path,
        graph: &mut DependencyGraph,
        profile_name: &str,
        owner_name: &str,
    ) -> Result<()> {
        for (dep_name, spec) in deps.iter() {
            if spec.is_system() {
                continue;
            }

            let constraint =
                DependencyConstraint::from_dependency_spec(dep_name, spec, owner_root)?;
            self.check_dependency_conflict(profile_name, dep_name, constraint, owner_name)?;

            let resolved_dep = self.resolve_dependency(dep_name, spec, owner_root, profile_name)?;
            let canonical_root = resolved_dep.root.clone();

            let dep_idx = graph.add_node(
                canonical_root.clone(),
                resolved_dep.config.clone(),
                resolved_dep.source.clone(),
                resolved_dep.checksum.clone(),
                resolved_dep.is_wheel,
                resolved_dep.build_flags.clone(),
            )?;

            graph.add_edge(owner_idx, dep_idx)?;

            if !graph.is_visiting(&canonical_root) {
                let dep_package_name = resolved_dep
                    .config
                    .package
                    .as_ref()
                    .map(|pkg| pkg.name.as_str())
                    .unwrap_or(dep_name);

                graph.mark_visiting(canonical_root.clone());
                self.visit_dependencies(
                    dep_idx,
                    &resolved_dep.config.dependencies,
                    &canonical_root,
                    graph,
                    profile_name,
                    dep_package_name,
                )?;
                graph.unmark_visiting(&canonical_root);
            }
        }
        Ok(())
    }
}