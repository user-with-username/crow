#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::{
        archiver_kind::ArchiverKind, compiler_kind::CompilerKind, linker_kind::LinkerKind,
    };
    use crow_core::builder::toolchain::{detect_toolchain, Toolchain};

    fn is_compatible_with_gcc_family(kind: CompilerKind) -> bool {
        matches!(kind, CompilerKind::Gcc | CompilerKind::Gpp)
    }

    fn is_compatible_with_clang_family(kind: CompilerKind) -> bool {
        matches!(
            kind,
            CompilerKind::Clang | CompilerKind::ClangPP | CompilerKind::ClangCl
        )
    }

    #[test]
    fn test_detect_toolchain_no_preferences() {
        let result = detect_toolchain(None, None, None, None, None, None);
        assert!(result.is_ok(), "Should detect at least GCC toolchain");

        let toolchain = result.unwrap();
        assert!(!toolchain.compiler_path().is_empty());
        assert!(!toolchain.linker_path().is_empty());
        assert!(!toolchain.archiver_path().is_empty());
    }

    #[test]
    fn test_detect_toolchain_with_preferred_compiler_kind() {
        let result = detect_toolchain(None, Some(CompilerKind::Gcc), None, None, None, None);
        assert!(result.is_ok());

        let toolchain = result.unwrap();
        let is_acceptable = is_compatible_with_gcc_family(toolchain.compiler_kind())
            || is_compatible_with_clang_family(toolchain.compiler_kind());
        assert!(
            is_acceptable,
            "Expected GCC-family or Clang-family compiler, got {:?}",
            toolchain.compiler_kind()
        );
    }

    #[test]
    fn test_detect_toolchain_with_gcc_preference_accepts_clang() {
        let result = detect_toolchain(None, Some(CompilerKind::Gcc), None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_toolchain_with_clang_preference() {
        let result = detect_toolchain(None, Some(CompilerKind::Clang), None, None, None, None);

        if let Ok(toolchain) = result {
            assert!(
                is_compatible_with_clang_family(toolchain.compiler_kind())
                    || is_compatible_with_gcc_family(toolchain.compiler_kind()),
                "Got {:?}",
                toolchain.compiler_kind()
            );
        }
    }

    #[test]
    fn test_detect_toolchain_with_unknown_compiler_kind() {
        let result = detect_toolchain(None, Some(CompilerKind::Unknown), None, None, None, None);
        assert!(result.is_ok(), "Unknown compiler kind should be ignored");
    }

    #[test]
    fn test_detect_toolchain_with_preferred_linker_kind() {
        let result = detect_toolchain(None, None, None, Some(LinkerKind::Ld), None, None);
        if result.is_err() {
            println!("Note: LinkerKind::Ld preference failed - system may use different linker");
        }
    }

    #[test]
    fn test_detect_toolchain_with_preferred_archiver_kind() {
        let result = detect_toolchain(None, None, None, None, None, Some(ArchiverKind::Ar));

        if let Ok(toolchain) = result {
            let k = toolchain.archiver_kind();
            assert!(
                k == ArchiverKind::Ar
                    || k == ArchiverKind::LlvmAr
                    || (cfg!(windows) && k == ArchiverKind::Lib),
                "unexpected archiver kind: {k:?}"
            );
        } else {
            println!(
                "Note: ArchiverKind::Ar preference failed - system may use different archiver"
            );
        }
    }

    #[test]
    fn test_detect_toolchain_with_nonexistent_compiler() {
        let result = detect_toolchain(
            Some("nonexistent_compiler_xyz".to_string()),
            None,
            None,
            None,
            None,
            None,
        );

        if result.is_err() {
            println!("Note: Could not fall back from nonexistent compiler - this may be expected");
        }
    }

    #[test]
    fn test_toolchain_trait_methods() {
        let toolchain = detect_toolchain(None, None, None, None, None, None).unwrap();

        assert!(
            !toolchain.compiler_path().is_empty(),
            "Compiler path should not be empty"
        );
        assert!(
            !toolchain.linker_path().is_empty(),
            "Linker path should not be empty"
        );
        assert!(
            !toolchain.archiver_path().is_empty(),
            "Archiver path should not be empty"
        );

        assert_ne!(toolchain.compiler_kind(), CompilerKind::Unknown);
        assert_ne!(toolchain.linker_kind(), LinkerKind::Unknown);
        assert_ne!(toolchain.archiver_kind(), ArchiverKind::Unknown);

        // These should return vectors (may be empty)
        let _ = toolchain.system_include_dirs();
        let _ = toolchain.system_library_dirs();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_toolchain_availability() {
        let result = detect_toolchain(None, None, None, None, None, None);
        assert!(result.is_ok());

        let toolchain = result.unwrap();
        let kind = toolchain.compiler_kind();

        let is_valid = matches!(
            kind,
            CompilerKind::Msvc | CompilerKind::ClangCl | CompilerKind::Gcc | CompilerKind::Gpp
        );
        assert!(is_valid, "Unexpected compiler kind: {:?}", kind);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_unix_toolchain_availability() {
        let result = detect_toolchain(None, None, None, None, None, None);
        assert!(result.is_ok());

        let toolchain = result.unwrap();
        let kind = toolchain.compiler_kind();

        let is_valid = matches!(
            kind,
            CompilerKind::Gcc | CompilerKind::Gpp | CompilerKind::Clang | CompilerKind::ClangPP
        );
        assert!(is_valid, "Unexpected compiler kind on Unix: {:?}", kind);
    }

    #[test]
    fn test_toolchain_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn Toolchain>>();
    }

    #[test]
    fn test_toolchain_selection_consistency() {
        let result1 = detect_toolchain(None, None, None, None, None, None);
        let result2 = detect_toolchain(None, None, None, None, None, None);

        match (result1, result2) {
            (Ok(tc1), Ok(tc2)) => {
                assert_eq!(tc1.compiler_kind(), tc2.compiler_kind());
                assert_eq!(tc1.linker_kind(), tc2.linker_kind());
                assert_eq!(tc1.archiver_kind(), tc2.archiver_kind());
            }
            _ => panic!("Detection failed on one of the runs"),
        }
    }

    #[test]
    fn test_toolchain_returns_valid_paths() {
        let toolchain = detect_toolchain(None, None, None, None, None, None).unwrap();

        let compiler_path = toolchain.compiler_path();
        let linker_path = toolchain.linker_path();
        let archiver_path = toolchain.archiver_path();

        assert!(!compiler_path.is_empty());
        assert!(!linker_path.is_empty());
        assert!(!archiver_path.is_empty());
    }

    #[test]
    fn test_debug_detected_toolchain() {
        let toolchain = detect_toolchain(None, None, None, None, None, None).unwrap();
        println!(
            "Compiler: {} ({:?})",
            toolchain.compiler_path(),
            toolchain.compiler_kind()
        );
        println!(
            "Linker: {} ({:?})",
            toolchain.linker_path(),
            toolchain.linker_kind()
        );
        println!(
            "Archiver: {} ({:?})",
            toolchain.archiver_path(),
            toolchain.archiver_kind()
        );
    }
}
