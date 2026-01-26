fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

fn fix_msvc_path(path: &str, default_extension: Option<&str>) -> String {
    let mut fixed = normalize_path(path);

    if let Some(ext) = default_extension {
        if !fixed.ends_with(ext) && !fixed.ends_with(&ext.to_uppercase()) {
            if fixed.contains('.') {
                if let Some(pos) = fixed.rfind('.') {
                    fixed = format!("{}{}", &fixed[..pos], ext);
                }
            } else {
                fixed.push_str(ext);
            }
        }
    }

    fixed
}
