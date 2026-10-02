use serde::Deserialize;

#[derive(Deserialize)]
pub struct TargetFile {
    pub name: String,
    pub r#type: String,
    #[serde(rename = "isImported")]
    pub is_imported: Option<bool>,
    #[serde(rename = "sourceDir")]
    pub source_dir: Option<String>,
    pub artifacts: Option<Vec<TargetArtifact>>,
    #[serde(rename = "compileGroups")]
    pub compile_groups: Option<Vec<TargetCompileGroup>>,
}

#[derive(Deserialize)]
pub struct TargetArtifact {
    pub path: String,
}

#[derive(Deserialize)]
pub struct TargetCompileGroup {
    pub includes: Option<Vec<TargetInclude>>,
}

#[derive(Deserialize)]
pub struct TargetInclude {
    pub path: String,
    #[serde(rename = "isSystem")]
    pub is_system: Option<bool>,
}