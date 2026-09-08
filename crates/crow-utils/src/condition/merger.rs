use super::evaluator::eval_condition;
use super::parser::parse_condition;
use super::target_info::TargetInfo;
use std::collections::HashMap;
use toml_edit::{Document, Item, Table};

fn merge_item(base: &mut Item, overlay: &Item) {
    match (base, overlay) {
        (Item::Table(base_tab), Item::Table(overlay_tab)) => {
            for (k, v) in overlay_tab.iter() {
                match base_tab.get_mut(k) {
                    Some(existing) => merge_item(existing, v),
                    None => {
                        base_tab.insert(k, v.clone());
                    }
                }
            }
        }
        (base_item, _) => *base_item = overlay.clone(),
    }
}

fn merge_table_into_root(root: &mut Document, section_name: &str, overlay: &Table) {
    let root_item = root
        .entry(section_name)
        .or_insert(Item::Table(Table::new()));
    if let Some(root_tab) = root_item.as_table_mut() {
        for (k, v) in overlay.iter() {
            match root_tab.get_mut(k) {
                Some(existing) => merge_item(existing, v),
                None => {
                    root_tab.insert(k, v.clone());
                }
            }
        }
    }
}

// Проверяет, является ли ключ условным таргетом (заключен в кавычки)
fn is_conditional_key(key: &str) -> bool {
    key.starts_with('"') && key.ends_with('"')
}

fn collect_conditional_sections(
    doc: &Document,
    target: &TargetInfo,
) -> Result<Vec<(String, Table)>, String> {
    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for (raw_key, value) in target_table.iter() {
        if !is_conditional_key(raw_key) {
            continue;
        }

        let cond_str = raw_key.trim_matches('"');

        let cond = parse_condition(cond_str)
            .map_err(|e| format!("failed to parse condition '{}': {}", cond_str, e))?;

        if !eval_condition(&cond, target) {
            continue;
        }

        if let Some(section_table) = value.as_table() {
            result.push((cond_str.to_string(), section_table.clone()));
        }
    }

    Ok(result)
}

fn apply_sections(doc: &mut Document, sections: Vec<(String, Table)>) {
    for (_cond_str, cond_table) in sections {
        for (section_name, section_value) in cond_table.iter() {
            match section_value.as_table() {
                Some(overlay_tab) => merge_table_into_root(doc, section_name, overlay_tab),
                None => {
                    *doc.entry(section_name).or_insert(section_value.clone()) =
                        section_value.clone();
                }
            }
        }
    }
}

// Применяет конкретный именованный таргет
fn apply_named_target(doc: &mut Document, target_config: &Table) {
    for (section_name, section_value) in target_config.iter() {
        match section_value.as_table() {
            Some(overlay_tab) => merge_table_into_root(doc, section_name, overlay_tab),
            None => {
                *doc.entry(section_name).or_insert(section_value.clone()) = section_value.clone();
            }
        }
    }
}

// Извлекает все именованные таргеты
fn get_named_targets(doc: &Document) -> HashMap<String, Table> {
    let mut named_targets = HashMap::new();

    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return named_targets,
    };

    for (key, value) in target_table.iter() {
        // Именованные таргеты - это ключи без кавычек
        if !is_conditional_key(key) {
            if let Some(section_table) = value.as_table() {
                named_targets.insert(key.to_string(), section_table.clone());
            }
        }
    }

    named_targets
}

pub fn apply_target_filter(content: &str, target_or_name: &str) -> Result<String, String> {
    let mut doc = content
        .parse::<Document>()
        .map_err(|e| format!("failed to parse TOML: {}", e))?;

    // Проверяем, есть ли у нас секция [target]
    let target_table = doc.get("target").and_then(|item| item.as_table());

    if target_table.is_some() {
        // Если target_or_name начинается и заканчивается на кавычку - это условный таргет
        if target_or_name.starts_with('"') && target_or_name.ends_with('"') {
            // Убираем кавычки и парсим как условие
            let condition = target_or_name.trim_matches('"');
            let fake_target_info = TargetInfo::current();
            let cond = parse_condition(condition)
                .map_err(|e| format!("failed to parse condition '{}': {}", condition, e))?;

            if eval_condition(&cond, &fake_target_info) {
                let conditional_sections = collect_conditional_sections(&doc, &fake_target_info)?;
                apply_sections(&mut doc, conditional_sections);
            }
        } else {
            // Иначе - это именованный таргет
            let named_targets = get_named_targets(&doc);
            if let Some(target_config) = named_targets.get(target_or_name) {
                apply_named_target(&mut doc, target_config);
            } else {
                return Err(format!(
                    "Named target '{}' not found in [target] section",
                    target_or_name
                ));
            }
        }
    }

    doc.remove("target");
    Ok(doc.to_string())
}

pub fn has_named_targets(content: &str) -> bool {
    let doc = match content.parse::<Document>() {
        Ok(doc) => doc,
        Err(_) => return false,
    };
    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return false,
    };
    for key in target_table.iter() {
        if !is_conditional_key(key.0) {
            return true;
        }
    }
    false
}

pub fn get_named_targets_list(content: &str) -> Vec<String> {
    let doc = match content.parse::<Document>() {
        Ok(doc) => doc,
        Err(_) => return Vec::new(),
    };
    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut targets = Vec::new();
    for key in target_table.iter() {
        if !is_conditional_key(key.0) {
            targets.push(key.0.to_string());
        }
    }
    targets
}
