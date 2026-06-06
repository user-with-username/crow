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
        self.include_dirs.extend(artifacts.include_dirs.iter().cloned());
        self.lib_paths.extend(artifacts.lib_paths.iter().cloned());
        self.libs.extend(artifacts.lib_names.iter().cloned());

        self.include_dirs.sort_unstable();
        self.include_dirs.dedup();

        self.lib_paths.sort_unstable();
        self.lib_paths.dedup();

        self.libs.sort();
        self.libs.dedup();
    }
}