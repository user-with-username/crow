#[cfg(test)]
mod tests {
    use crow_utils::helpers::{change_directory, find_executable, fix_msvc_path, normalize_path};
    use tempfile::tempdir;
    use std::env;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_change_directory_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let original_dir = env::current_dir()?;
        
        change_directory(temp_dir.path())?;
        assert_eq!(env::current_dir()?, temp_dir.path());
        
        change_directory(&original_dir)?;
        Ok(())
    }

    #[test]
    fn test_change_directory_failure() {
        let result = change_directory(Path::new("/nonexistent/directory/that/doesnt/exist"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to change directory"));
    }

    #[test]
    fn test_normalize_path_windows_remove_extended_prefix() {
        if cfg!(windows) {
            assert_eq!(normalize_path(r"\\?\C:\Windows\System32"), r"C:\Windows\System32");
            assert_eq!(normalize_path(r"\\?\UNC\server\share"), r"\\server\share");
            assert_eq!(normalize_path(r"\\?\C:\test\file.txt"), r"C:\test\file.txt");
        }
    }

    #[test]
    fn test_normalize_path_windows_unc() {
        if cfg!(windows) {
            assert_eq!(normalize_path(r"UNC\server\share"), r"\\server\share");
            assert_eq!(normalize_path(r"UNC\long\path\to\file"), r"\\long\path\to\file");
        }
    }

    #[test]
    fn test_normalize_path_unix_backslashes() {
        if !cfg!(windows) {
            assert_eq!(normalize_path(r"path\to\file"), "path/to/file");
            assert_eq!(normalize_path(r"C:\windows\style"), "C:/windows/style");
            assert_eq!(normalize_path(r"back\slash\mix/here"), "back/slash/mix/here");
        }
    }

    #[test]
    fn test_normalize_path_windows_forward_slashes() {
        if cfg!(windows) {
            assert_eq!(normalize_path("path/to/file"), r"path\to\file");
            assert_eq!(normalize_path("unix/style/path"), r"unix\style\path");
            assert_eq!(normalize_path("C:/Program Files/App"), r"C:\Program Files\App");
        }
    }

    #[test]
    fn test_normalize_path_preserves_unchanged() {
        if cfg!(windows) {
            assert_eq!(normalize_path(r"C:\Windows\System32"), r"C:\Windows\System32");
            assert_eq!(normalize_path(r"relative\path"), r"relative\path");
        } else {
            assert_eq!(normalize_path("/usr/local/bin"), "/usr/local/bin");
            assert_eq!(normalize_path("relative/path"), "relative/path");
        }
    }

    #[test]
    fn test_fix_msvc_path_add_extension() {
        assert_eq!(
            fix_msvc_path("output", Some(".exe")),
            if cfg!(windows) { "output.exe" } else { "output.exe" }
        );
        
        assert_eq!(
            fix_msvc_path("file.obj", Some(".exe")),
            if cfg!(windows) { "file.exe" } else { "file.exe" }
        );
    }

    #[test]
    fn test_fix_msvc_path_preserve_existing_extension() {
        assert_eq!(
            fix_msvc_path("program.exe", Some(".exe")),
            if cfg!(windows) { "program.exe" } else { "program.exe" }
        );
        
        assert_eq!(
            fix_msvc_path("library.dll", Some(".exe")),
            if cfg!(windows) { "library.exe" } else { "library.exe" }
        );
    }

    #[test]
    fn test_fix_msvc_path_multiple_dots() {
        assert_eq!(
            fix_msvc_path("archive.tar.gz", Some(".exe")),
            if cfg!(windows) { "archive.tar.exe" } else { "archive.tar.exe" }
        );
        
        assert_eq!(
            fix_msvc_path("file.name.with.dots.txt", Some(".exe")),
            if cfg!(windows) { "file.name.with.dots.exe" } else { "file.name.with.dots.exe" }
        );
    }

    #[test]
    fn test_fix_msvc_path_no_extension() {
        assert_eq!(
            fix_msvc_path("myprogram", None),
            if cfg!(windows) { "myprogram" } else { "myprogram" }
        );
        
        assert_eq!(
            fix_msvc_path("path/to/program", None),
            if cfg!(windows) { "path\\to\\program" } else { "path/to/program" }
        );
    }

    #[test]
    fn test_find_executable_not_found() {
        let result = find_executable("nonexistent_executable_xyz_123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[cfg(windows)]
    fn test_find_executable_windows_common_locations() {
        let result = find_executable("powershell");
        if result.is_ok() {
            let path = result.unwrap();
            assert!(path.to_string_lossy().contains("PowerShell") || 
                    path.to_string_lossy().contains("powershell"));
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_find_executable_macos_common_locations() -> Result<(), Box<dyn std::error::Error>> {
        let result = find_executable("ls")?;
        assert_eq!(result, PathBuf::from("/bin/ls"));
        Ok(())
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_find_executable_linux_common_locations() -> Result<(), Box<dyn std::error::Error>> {
        let result = find_executable("ls")?;
        assert!(result == PathBuf::from("/bin/ls") || result == PathBuf::from("/usr/bin/ls"));
        Ok(())
    }

    #[test]
    fn test_normalize_and_fix_combined() {
        let path = "C:/Program Files/MyApp/bin/tool";
        let normalized = normalize_path(path);
        
        if cfg!(windows) {
            assert_eq!(normalized, r"C:\Program Files\MyApp\bin\tool");
            
            let fixed = fix_msvc_path(&normalized, Some(".exe"));
            assert_eq!(fixed, r"C:\Program Files\MyApp\bin\tool.exe");
        } else {
            assert_eq!(normalized, "C:/Program Files/MyApp/bin/tool");
            
            let fixed = fix_msvc_path(&normalized, Some(".exe"));
            assert_eq!(fixed, "C:/Program Files/MyApp/bin/tool.exe");
        }
    }

    #[test]
    fn test_find_executable_with_custom_path() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let exe_name = "my_custom_tool";
        let exe_path = if cfg!(windows) {
            temp_dir.path().join(format!("{}.exe", exe_name))
        } else {
            temp_dir.path().join(exe_name)
        };
        
        fs::write(&exe_path, "")?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms)?;
        }
        
        let original_path = env::var_os("PATH").unwrap_or_default();
        env::set_var("PATH", temp_dir.path());
        
        let result = find_executable(exe_name)?;
        env::set_var("PATH", original_path);
        
        assert_eq!(result, exe_path);
        
        Ok(())
    }

    // Тест для проверки работы с UNC путями на Windows
    #[test]
    #[cfg(windows)]
    fn test_normalize_path_unc_complex() {
        assert_eq!(
            normalize_path(r"\\?\UNC\server\share\folder\file.txt"),
            r"\\server\share\folder\file.txt"
        );
        
        assert_eq!(
            normalize_path(r"UNC\server\share\folder\file.txt"),
            r"\\server\share\folder\file.txt"
        );
    }
}