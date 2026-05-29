use crate::condition::CfgEvaluator;
use toml::value::{Table, Value};

pub fn apply_target_conditions(table: &mut Table) {
    let evaluator = CfgEvaluator::new();
    apply_target_conditions_with(table, &evaluator)
}

pub fn apply_target_conditions_with(table: &mut Table, evaluator: &CfgEvaluator) {
    let mut matches: Vec<(Vec<String>, Value)> = Vec::new();

    if let Some(Value::Table(target_table)) = table.remove("target") {
        collect_target_entries(target_table, &mut matches, evaluator);
    }

    for (path, value) in &matches {
        merge_at_path(table, path, value);
    }
}

fn collect_target_entries(
    table: Table,
    out: &mut Vec<(Vec<String>, Value)>,
    evaluator: &CfgEvaluator,
) {
    for (key, value) in table {
        if is_condition_key(&key) {
            if let Value::Table(inner) = value {
                if evaluator.eval(&key) {
                    collect_leaves(inner, &mut Vec::new(), out);
                }
            }
        }
    }
}

fn is_condition_key(key: &str) -> bool {
    let k = key.trim();
    k.starts_with("cfg(") || k.starts_with("all(") || k.starts_with("any(") || k.starts_with("not(")
}

fn collect_leaves(table: Table, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, Value)>) {
    for (key, value) in table {
        match value {
            Value::Table(inner) => {
                prefix.push(key);
                collect_leaves(inner, prefix, out);
                prefix.pop();
            }
            _ => {
                let mut path = prefix.clone();
                path.push(key);
                out.push((path, value));
            }
        }
    }
}

fn merge_at_path(table: &mut Table, path: &[String], value: &Value) {
    if path.is_empty() {
        return;
    }

    if path.len() == 1 {
        let key = &path[0];
        match table.get_mut(key) {
            Some(Value::Table(existing)) => {
                if let Value::Table(src) = value {
                    deep_merge_table(existing, src);
                } else {
                    table.insert(key.clone(), value.clone());
                }
            }
            Some(_) => {
                table.insert(key.clone(), value.clone());
            }
            None => {
                table.insert(key.clone(), value.clone());
            }
        }
    } else {
        let key = &path[0];
        let sub = table
            .entry(key.clone())
            .or_insert_with(|| Value::Table(Table::new()));
        if let Value::Table(sub_table) = sub {
            merge_at_path(sub_table, &path[1..], value);
        } else {
            let mut new_sub = Table::new();
            merge_at_path(&mut new_sub, &path[1..], value);
            table.insert(key.clone(), Value::Table(new_sub));
        }
    }
}

fn deep_merge_table(dst: &mut Table, src: &Table) {
    for (key, src_val) in src {
        match dst.get_mut(key) {
            Some(Value::Table(dst_sub)) => {
                if let Value::Table(src_sub) = src_val {
                    deep_merge_table(dst_sub, src_sub);
                } else {
                    dst.insert(key.clone(), src_val.clone());
                }
            }
            _ => {
                dst.insert(key.clone(), src_val.clone());
            }
        }
    }
}
