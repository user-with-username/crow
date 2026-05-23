#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::archiver_kind::ArchiverKind;
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use crow_core::builder::toolchain::{GccToolchain, Toolchain};

    fn print_result_error(result: &anyhowed::Result<GccToolchain>) {
        match result {
            Ok(_) => println!("Result: Ok(GccToolchain)"),
            Err(e) => println!("Result: Err({})", e),
        }
    }

    #[test]
    fn test_detect_with_no_preferences() {
        let result = GccToolchain::detect(None, None, None, None);
        assert!(result.is_ok(), "Should detect a toolchain");

        let toolchain = result.unwrap();
        assert!(!toolchain.compiler_path().is_empty());
        assert!(!toolchain.linker_path().is_empty());
        assert!(!toolchain.archiver_path().is_empty());
        assert_ne!(toolchain.compiler_kind(), CompilerKind::Unknown);
        assert_ne!(toolchain.archiver_kind(), ArchiverKind::Unknown);
    }

    #[test]
    fn test_detect_with_preferred_linker() {
        let result = GccToolchain::detect(None, Some("ld".to_string()), None, None);
        if let Ok(toolchain) = result {
            assert!(!toolchain.linker_path().is_empty());
        }
    }

    #[test]
    fn test_detect_with_preferred_archiver() {
        let result =
            GccToolchain::detect(None, None, Some("ar".to_string()), Some(ArchiverKind::Ar));

        if let Ok(toolchain) = result {
            assert_eq!(toolchain.archiver_path(), "ar");
            assert_eq!(toolchain.archiver_kind(), ArchiverKind::Ar);
        }
    }

    #[test]
    fn test_toolchain_trait_methods() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let _compiler_path = toolchain.compiler_path();
            let _compiler_kind = toolchain.compiler_kind();
            let _linker_path = toolchain.linker_path();
            let _linker_kind = toolchain.linker_kind();
            let _archiver_path = toolchain.archiver_path();
            let _archiver_kind = toolchain.archiver_kind();
            let _include_dirs = toolchain.system_include_dirs();
            let _library_dirs = toolchain.system_library_dirs();

            assert!(!_compiler_path.is_empty());
            assert!(!_linker_path.is_empty());
            assert!(!_archiver_path.is_empty());
        }
    }

    #[test]
    fn test_linker_kind_mapping() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let kind = toolchain.linker_kind();
            println!("Detected linker kind: {:?}", kind);
        }
    }

    #[test]
    fn test_system_directories() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let includes = toolchain.system_include_dirs();
            let libs = toolchain.system_library_dirs();
            println!("System includes count: {}", includes.len());
            println!("System libraries count: {}", libs.len());
        }
    }

    #[test]
    fn test_identify_linker_public() {
        let is_linker = GccToolchain::identify_linker("ld");
        println!("ld identified as linker: {}", is_linker);
    }

    #[test]
    fn test_detect_with_nonexistent_compiler() {
        let result = GccToolchain::detect(
            Some("nonexistent_compiler_xyz".to_string()),
            None,
            None,
            None,
        );

        assert!(result.is_err());
        if let Err(e) = result {
            println!("Expected error: {}", e);
        }
    }

    #[test]
    fn test_detect_with_incompatible_archiver() {
        let result = GccToolchain::detect(
            None,
            None,
            Some("lib.exe".to_string()),
            Some(ArchiverKind::Lib),
        );

        print_result_error(&result);
    }

    #[test]
    fn test_toolchain_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GccToolchain>();
        assert_send_sync::<Box<dyn Toolchain>>();
    }

    #[test]
    fn test_consistent_detection() {
        let result1 = GccToolchain::detect(None, None, None, None);
        let result2 = GccToolchain::detect(None, None, None, None);

        match (&result1, &result2) {
            (Ok(tc1), Ok(tc2)) => {
                assert_eq!(tc1.compiler_kind(), tc2.compiler_kind());
                assert_eq!(tc1.linker_kind(), tc2.linker_kind());
                assert_eq!(tc1.archiver_kind(), tc2.archiver_kind());
            }
            (Err(_), Err(_)) => {
                assert!(result1.is_err() && result2.is_err());
            }
            _ => {
                panic!("Inconsistent detection: one succeeded, one failed");
            }
        }
    }

    #[test]
    fn test_detect_returns_some_compiler() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let kind = toolchain.compiler_kind();
            let is_known = matches!(
                kind,
                CompilerKind::Gcc
                    | CompilerKind::Gpp
                    | CompilerKind::Clang
                    | CompilerKind::ClangPP
                    | CompilerKind::ClangCl
                    | CompilerKind::Msvc
            );
            assert!(
                is_known,
                "Should detect a known compiler kind, got {:?}",
                kind
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_unix_detection() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let kind = toolchain.compiler_kind();
            let is_valid = matches!(
                kind,
                CompilerKind::Gcc | CompilerKind::Gpp | CompilerKind::Clang | CompilerKind::ClangPP
            );
            assert!(is_valid, "Unexpected compiler kind on Unix: {:?}", kind);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_detection() {
        let result = GccToolchain::detect(None, None, None, None);
        if let Ok(toolchain) = result {
            let kind = toolchain.compiler_kind();
            println!("Windows compiler kind: {:?}", kind);
        }
    }
}
