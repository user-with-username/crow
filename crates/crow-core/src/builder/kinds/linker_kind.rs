use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkerKind {
    Ld,
    Lld,
    Link,
    AppleLd,
    WasmLd,
    Gold,
    BpfLink,
    Unknown,
}
