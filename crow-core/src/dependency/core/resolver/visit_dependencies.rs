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
        owner_root: &Path,
        graph: &mut DependencyGraph,
        profile_name: &str,
        owner_name: &str,
    ) -> Result<()> {
        for (dep_name, spec) in deps.iter() {
            if spec.is_system() {
                continue;
            }

            let constraint = DependencyConstraint::from_dependency_spec(spec, owner_root);
            self.check_dependency_conflict(profile_name, dep_name, constraint, owner_name)?;

            let resolved_dep = self.resolve_dependency(dep_name, spec, owner_root, profile_name)?;
            
            let crate::dependency::ResolvedPackage {
                root: canonical_root,
                config,
                source,
                checksum,
                is_wheel,
                build_flags,
                name: _, 
            } = resolved_dep;

            let sub_dependencies = config.dependencies.clone();

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
            )?;

            graph.add_edge(owner_idx, dep_idx)?;

            if !graph.is_visiting(&canonical_root) {
                graph.mark_visiting(canonical_root.clone());
                
                self.visit_dependencies(
                    dep_idx,
                    &sub_dependencies,
                    &canonical_root,
                    graph,
                    profile_name,
                    &dep_package_name,
                )?;
                
                graph.unmark_visiting(&canonical_root);
            }
        }
        Ok(())
    }
}