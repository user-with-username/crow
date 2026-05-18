#[cfg(test)]
mod tests {
    use std::fs;

    use crow_cli::templates::{create_project_structure, CROW_TOML_TEMPLATE, GITIGNORE, MAIN_CPP};
    use tempfile::tempdir;

    #[test]
    fn test_create_project_structure_creates_all_files() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        let package_name = "test-project";

        create_project_structure(base_path, package_name, false).unwrap();

        let main_cpp_path = base_path.join("src/main.cpp");
        assert!(main_cpp_path.exists());
        assert_eq!(fs::read_to_string(main_cpp_path).unwrap(), MAIN_CPP);

        let crow_toml_path = base_path.join("crow.toml");
        assert!(crow_toml_path.exists());
        let expected_toml = CROW_TOML_TEMPLATE.replace("{name}", package_name);
        assert_eq!(fs::read_to_string(crow_toml_path).unwrap(), expected_toml);

        let gitignore_path = base_path.join(".gitignore");
        assert!(gitignore_path.exists());
        assert_eq!(fs::read_to_string(gitignore_path).unwrap(), GITIGNORE);
    }

    #[test]
    fn test_create_project_structure_no_overwrite_existing() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        let package_name = "test-project";

        create_project_structure(base_path, package_name, false).unwrap();

        let modified_content = "modified content";
        fs::write(base_path.join("crow.toml"), modified_content).unwrap();

        create_project_structure(base_path, package_name, false).unwrap();

        assert_eq!(
            fs::read_to_string(base_path.join("crow.toml")).unwrap(),
            modified_content
        );
    }

    #[test]
    fn test_create_project_structure_with_overwrite() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        let package_name = "test-project";

        create_project_structure(base_path, package_name, false).unwrap();

        fs::write(base_path.join("crow.toml"), "modified content").unwrap();

        create_project_structure(base_path, package_name, true).unwrap();

        let expected_toml = CROW_TOML_TEMPLATE.replace("{name}", package_name);
        assert_eq!(
            fs::read_to_string(base_path.join("crow.toml")).unwrap(),
            expected_toml
        );
    }

    #[test]
    fn test_create_project_structure_creates_parent_directories() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().join("nested").join("deep").join("path");
        let package_name = "test-project";

        create_project_structure(&base_path, package_name, false).unwrap();

        assert!(base_path.join("src/main.cpp").exists());
        assert!(base_path.join("crow.toml").exists());
        assert!(base_path.join(".gitignore").exists());
    }
}
