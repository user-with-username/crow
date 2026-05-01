use crate::config::CrowConfig;
use anyhow::{anyhow, Result};
use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NodePayload {
    pub root: PathBuf,
    pub config: CrowConfig,
    pub source: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: Graph<NodePayload, ()>,
    node_by_root: HashMap<PathBuf, NodeIndex>,
    visiting: HashSet<PathBuf>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_by_root: HashMap::new(),
            visiting: HashSet::new(),
        }
    }

    pub fn add_node(
        &mut self,
        root: PathBuf,
        config: CrowConfig,
        source: Option<String>,
        checksum: Option<String>,
    ) -> Result<NodeIndex> {
        if let Some(&existing) = self.node_by_root.get(&root) {
            return Ok(existing);
        }

        let idx = self.graph.add_node(NodePayload {
            root: root.clone(),
            config,
            source,
            checksum,
        });
        self.node_by_root.insert(root, idx);
        Ok(idx)
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) -> Result<()> {
        if self.graph.find_edge(from, to).is_none() {
            self.graph.add_edge(from, to, ());
        }
        Ok(())
    }

    pub fn get_node(&self, idx: NodeIndex) -> Option<&NodePayload> {
        self.graph.node_weight(idx)
    }

    pub fn resolve_order(&self) -> Result<Vec<NodeIndex>> {
        let sorted = toposort(&self.graph, None).map_err(|cycle| {
            let idx = cycle.node_id();
            let package_name = self.graph[idx]
                .config
                .package
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| self.graph[idx].root.display().to_string());
            anyhow!("dependency cycle detected near package `{package_name}`")
        })?;

        let mut build_order = sorted;
        build_order.reverse();
        Ok(build_order)
    }

    pub fn is_visiting(&self, root: &PathBuf) -> bool {
        self.visiting.contains(root)
    }

    pub fn mark_visiting(&mut self, root: PathBuf) {
        self.visiting.insert(root);
    }

    pub fn unmark_visiting(&mut self, root: &PathBuf) {
        self.visiting.remove(root);
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}