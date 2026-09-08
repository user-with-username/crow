mod evaluator;
mod merger;
mod parser;
mod target_info;

pub use evaluator::eval_condition;
pub use merger::{apply_target_filter, get_named_targets_list, has_named_targets};
pub use parser::parse_condition;
pub use target_info::TargetInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Cfg { key: String, value: Option<String> },
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    Triple(String),
}
