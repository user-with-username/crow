use super::evaluator::eval_condition;
use super::parser::parse_condition;
use super::target_info::TargetInfo;
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

fn collect_matching_sections(
    doc: &Document,
    target: &TargetInfo,
) -> Result<Vec<(String, Table)>, String> {
    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for (raw_key, value) in target_table.iter() {
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

fn apply_matching_sections(doc: &mut Document, sections: Vec<(String, Table)>) {
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

pub fn apply_target_filter(content: &str, target: &TargetInfo) -> Result<String, String> {
    let mut doc = content
        .parse::<Document>()
        .map_err(|e| format!("failed to parse TOML: {}", e))?;

    let matching = collect_matching_sections(&doc, target)?;
    apply_matching_sections(&mut doc, matching);
    doc.remove("target");

    Ok(doc.to_string())
}
