use crate::config::CrowConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub root: PathBuf,
    pub config: CrowConfig,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub is_wheel: bool,
    pub build_flags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedDependencyBuild {
    pub packages: Vec<ResolvedPackage>,
    pub include_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
    pub lib_paths: Vec<PathBuf>,
    pub max_standard: Option<String>,
    pub system_libs: Vec<String>,
}

impl ResolvedDependencyBuild {
    pub fn absorb_wheel_artifacts(&mut self, artifacts: &crate::dependency::WheelArtifacts) {
        use std::collections::HashSet;

        let mut seen_include: HashSet<PathBuf> = self.include_dirs.iter().cloned().collect();
        let mut seen_libs: HashSet<String> = self.libs.iter().cloned().collect();
        let mut seen_lib_paths: HashSet<PathBuf> = self.lib_paths.iter().cloned().collect();

        for include_dir in &artifacts.include_dirs {
            if seen_include.insert(include_dir.clone()) {
                self.include_dirs.push(include_dir.clone());
            }
        }
        for lib_path in &artifacts.lib_paths {
            if seen_lib_paths.insert(lib_path.clone()) {
                self.lib_paths.push(lib_path.clone());
            }
        }
        for lib_name in &artifacts.lib_names {
            if seen_libs.insert(lib_name.clone()) {
                self.libs.push(lib_name.clone());
            }
        }
    }
}
