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
}

#[derive(Deserialize)]
pub struct TargetArtifact {
    pub path: String,
}