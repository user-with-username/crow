#[cfg(test)]
mod tests {
    use crow_utils::msvc::{determine_target_arch, find_vswhere};
    use std::{env, path::PathBuf};

    #[test]
    fn test_determine_target_arch_from_env() {
        env::set_var("TARGET", "x86_64-pc-windows-msvc");
        let path = PathBuf::from("dummy.exe");
        assert_eq!(determine_target_arch(&path), "x64");

        env::set_var("TARGET", "i686-pc-windows-msvc");
        assert_eq!(determine_target_arch(&path), "x86");

        env::remove_var("TARGET");
    }

    #[test]
    fn test_find_vswhere() {
        let result = find_vswhere();
        if cfg!(windows) {
            // mb or mb not exist on ci, so just test it runs
            let _ = result;
        } else {
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_version_sorting() {
        let mut versions = vec![
            "14.38.33130".to_string(),
            "14.9.12345".to_string(),
            "14.10.0".to_string(),
        ];
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
            a_parts.cmp(&b_parts)
        });
        assert_eq!(versions, vec!["14.9.12345", "14.10.0", "14.38.33130"]);
    }
}
