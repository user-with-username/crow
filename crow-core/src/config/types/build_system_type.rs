use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum BuildSystemType {
    #[serde(rename = "crow")]
    Crow,
    #[serde(rename = "cmake")]
    Cmake,
}
