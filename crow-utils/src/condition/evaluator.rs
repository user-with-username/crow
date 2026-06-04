use crate::condition::Condition;
use super::target_info::TargetInfo;

fn eval_cfg(key: &str, value: Option<&str>, target: &TargetInfo) -> bool {
    match (key, value) {
        ("windows", None) => target.is_windows(),
        ("unix", None) => target.is_unix(),
        ("macos", None) => target.is_macos(),
        ("linux", None) => target.is_linux(),
        ("target_os", Some(v)) => v == target.os,
        ("target_arch", Some(v)) => v == target.arch,
        ("target_vendor", Some(v)) => v == target.vendor,
        ("target_env", Some(v)) => v == target.env,
        ("target_pointer_width", Some(v)) => v == target.pointer_width.to_string(),
        ("target_endian", Some(v)) => {
            if cfg!(target_endian = "little") { v == "little" } else { v == "big" }
        }
        ("target_family", Some(v)) => target.family.as_deref() == Some(v),
        _ => false,
    }
}

pub fn eval_condition(cond: &Condition, target: &TargetInfo) -> bool {
    match cond {
        Condition::Cfg { key, value } => eval_cfg(key, value.as_deref(), target),
        Condition::All(conds) => conds.iter().all(|c| eval_condition(c, target)),
        Condition::Any(conds) => conds.iter().any(|c| eval_condition(c, target)),
        Condition::Not(c) => !eval_condition(c, target),
        Condition::Triple(triple_str) => target.triple.to_string() == *triple_str,
    }
}