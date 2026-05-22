mod git;
mod graph;
mod lockfile;
mod merge;
mod registry;
pub mod version_req;
mod wheel;
pub mod core;

pub use git::GitDependencyFetcher;
pub use graph::DependencyGraph;
pub use lockfile::LockfileBuilder;
pub use merge::{apply_dependency_standard, format_lock_dependencies, merge_dependency_inputs};
pub use registry::RegistryFetcher;
pub use version_req::VersionReq;
pub use wheel::{create_wheel, WheelArtifacts, WheelType};
pub use core::{DependencyResolver, ResolvedDependencyBuild, ResolvedPackage};