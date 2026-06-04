#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Cfg { key: String, value: Option<String> },
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    Triple(String),
}
