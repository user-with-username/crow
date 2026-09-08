use crate::config::CrowConfig;
use anyhowed::{anyhow, Result};
use colored::Colorize;
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
    pub is_wheel: bool,
    pub build_flags: Vec<String>,
    /// Features requested from a registry dependency (propagated from ResolvedPackage).
    pub registry_features: Vec<String>,
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: Graph<NodePayload, ()>,
    node_by_name: HashMap<String, NodeIndex>,
    visiting: HashSet<PathBuf>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_by_name: HashMap::new(),
            visiting: HashSet::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_node(
        &mut self,
        root: PathBuf,
        config: CrowConfig,
        source: Option<String>,
        checksum: Option<String>,
        is_wheel: bool,
        build_flags: Vec<String>,
        registry_features: Vec<String>,
    ) -> Result<NodeIndex> {
        let name = config
            .package
            .as_ref()
            .map(|pkg| pkg.name.clone())
            .unwrap_or_else(|| {
                root.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        if let Some(&existing_idx) = self.node_by_name.get(&name) {
            return Ok(existing_idx);
        }

        let idx = self.graph.add_node(NodePayload {
            root,
            config,
            source,
            checksum,
            is_wheel,
            build_flags,
            registry_features,
        });
        self.node_by_name.insert(name, idx);
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

    /// Returns the name of the root node (first added node), or None if empty.
    pub fn root_name(&self) -> Option<String> {
        self.graph.node_indices().next().and_then(|idx| {
            self.graph.node_weight(idx).map(|p| {
                p.config
                    .package
                    .as_ref()
                    .map(|pkg| pkg.name.clone())
                    .unwrap_or_else(|| {
                        p.root
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
            })
        })
    }

    /// Returns the names of all children of the node identified by `name`.
    pub fn children(&self, name: &str) -> Vec<String> {
        let idx = match self.node_by_name.get(name) {
            Some(idx) => *idx,
            None => return Vec::new(),
        };
        self.graph
            .neighbors(idx)
            .filter_map(|child_idx| {
                self.graph.node_weight(child_idx).map(|p| {
                    p.config
                        .package
                        .as_ref()
                        .map(|pkg| pkg.name.clone())
                        .unwrap_or_else(|| {
                            p.root
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        })
                })
            })
            .collect()
    }

    /// Returns the full label (name + version) for a node, or None if not found.
    pub fn label(&self, name: &str) -> Option<String> {
        self.node_by_name.get(name).and_then(|idx| {
            self.graph.node_weight(*idx).map(|p| {
                let pkg_name = p
                    .config
                    .package
                    .as_ref()
                    .map(|pkg| pkg.name.clone())
                    .unwrap_or_else(|| {
                        p.root
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    });
                let version = p
                    .config
                    .package
                    .as_ref()
                    .map(|pkg| pkg.version.clone())
                    .unwrap_or_default();
                if version.is_empty() {
                    pkg_name
                } else {
                    format!("{} {}", pkg_name, version)
                }
            })
        })
    }

    /// Returns true if the node has outgoing dependency edges.
    pub fn has_children(&self, name: &str) -> bool {
        self.node_by_name
            .get(name)
            .map(|idx| self.graph.neighbors(*idx).count() > 0)
            .unwrap_or(false)
    }

    /// Prints the dependency tree starting from the root node, matching cargo tree's output style.
    pub fn print_tree(&self, depth: usize, no_dedupe: bool) {
        let root_name = match self.root_name() {
            Some(name) => name,
            None => return,
        };
        let mut visited: HashSet<String> = HashSet::new();
        let mut print_stack: Vec<String> = Vec::new();
        let mut levels_continue: Vec<bool> = Vec::new();
        Self::print_node(
            self,
            &root_name,
            0,
            depth,
            no_dedupe,
            &mut visited,
            &mut levels_continue,
            &mut print_stack,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn print_node(
        &self,
        name: &str,
        depth: usize,
        max_depth: usize,
        no_dedupe: bool,
        visited: &mut HashSet<String>,
        levels_continue: &mut Vec<bool>,
        print_stack: &mut Vec<String>,
    ) {
        let label = self.label(name).unwrap_or_else(|| name.to_string());
        let is_duplicate = visited.contains(&label);
        if !no_dedupe {
            visited.insert(label.clone());
        }

        let in_cycle = print_stack.contains(&label);
        let has_children = self.has_children(name);

        let prefix = if depth > 0 {
            let mut prefix_str = String::new();
            for &continues in levels_continue.iter() {
                let c = if continues { "│" } else { " " };
                prefix_str.push_str(c);
                prefix_str.push_str("   ");
            }
            let c = if levels_continue.last().copied().unwrap_or(false) {
                "├"
            } else {
                "└"
            };
            prefix_str.push_str(c);
            prefix_str.push_str("──");
            prefix_str
        } else {
            String::new()
        };

        let parts: Vec<&str> = label.splitn(2, ' ').collect();
        let colored_label = if parts.len() == 2 {
            format!("{} {}", parts[0].bold(), parts[1].dimmed())
        } else {
            label.bold().to_string()
        };

        let star = if (!is_duplicate && !in_cycle) || !has_children {
            ""
        } else {
            " (*)"
        };
        println!("{}{}{}", prefix, colored_label, star.yellow().dimmed());

        if !no_dedupe && (is_duplicate || in_cycle) {
            return;
        }

        print_stack.push(label);

        if depth >= max_depth {
            print_stack.pop();
            return;
        }

        let mut child_names = self.children(name);
        child_names.sort();

        for (i, child_name) in child_names.iter().enumerate() {
            let is_last = i == child_names.len() - 1;
            levels_continue.push(!is_last);
            Self::print_node(
                self,
                child_name,
                depth + 1,
                max_depth,
                no_dedupe,
                visited,
                levels_continue,
                print_stack,
            );
            levels_continue.pop();
        }

        print_stack.pop();
    }

    pub fn graph(&self) -> &Graph<NodePayload, ()> {
        &self.graph
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
