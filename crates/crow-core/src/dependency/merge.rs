use crate::config::CrowConfig;
use crate::config::Dependencies;
use crate::dependency::ResolvedDependencyBuild;
use crate::dependency::ResolvedPackage;
use std::collections::{HashMap, HashSet};

pub fn merge_dependency_inputs(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
    let mut include_seen: HashSet<_> = config.build.include_dirs.iter().cloned().collect();
    for include_dir in &resolved.include_dirs {
        if include_seen.insert(include_dir.clone()) {
            config.build.include_dirs.push(include_dir.clone());
        }
    }

    let mut libs_seen: HashSet<_> = config.build.libs.iter().cloned().collect();
    for lib in &resolved.libs {
        if libs_seen.insert(lib.clone()) {
            config.build.libs.push(lib.clone());
        }
    }

    let mut lib_paths_seen: HashSet<_> = config.build.lib_dirs.iter().cloned().collect();
    for lib_path in &resolved.lib_paths {
        if lib_paths_seen.insert(lib_path.clone()) {
            config.build.lib_dirs.push(lib_path.clone());
        }
    }
}

pub fn apply_dependency_standard(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
    if let Some(max_standard) = &resolved.max_standard {
        if let Some(package) = config.package.as_mut() {
            package.standard = Some(max_standard.clone());
        }
    }
}

pub fn format_lock_dependencies(
    dependencies: &Dependencies,
    by_name: &HashMap<String, &ResolvedPackage>,
) -> Vec<String> {
    dependencies
        .keys()
        .map(|dep_name| {
            if let Some(dep) = by_name.get(dep_name) {
                let mut entry = format!(
                    "{} {}",
                    dep.name,
                    dep.config
                        .package
                        .as_ref()
                        .map(|pkg| pkg.version.as_str())
                        .unwrap_or("0.0.0")
                );
                if let Some(source) = &dep.source {
                    entry.push_str(&format!(" ({source})"));
                }
                entry
            } else {
                dep_name.clone()
            }
        })
        .collect()
}
