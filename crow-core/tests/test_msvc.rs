#[cfg(test)]
mod tests {
use crow_core::builder::toolchain::{MsvcToolchain, Toolchain};
#[cfg(target_os = "windows")]
use crow_core::builder::kinds::archiver_kind::ArchiverKind;
#[cfg(target_os = "windows")]
use crow_core::builder::kinds::compiler_kind::CompilerKind;
#[cfg(target_os = "windows")]
use crow_core::builder::kinds::linker_kind::LinkerKind;
#[cfg(target_os = "windows")]
use std::process::Command;

    #[cfg(target_os = "windows")]
    fn print_result_error(result: &anyhow::Result<MsvcToolchain>) {
        match result {
            Ok(_) => println!("Result: Ok(MsvcToolchain)"),
            Err(e) => println!("Result: Err({})", e),
        }
    }

    #[test]
    fn test_msvc_only_available_on_windows() {
        let result = MsvcToolchain::detect(None, None, None, None);
        
        #[cfg(target_os = "windows")]
        {
            println!("MSVC detection result on Windows:");
            print_result_error(&result);
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("only available on Windows"));
            }
        }
    }


    #[test]
    fn test_detect_with_no_preferences() {
        let result = MsvcToolchain::detect(None, None, None, None);
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(toolchain) = result {
                assert!(!toolchain.compiler_path().is_empty());
                assert!(!toolchain.linker_path().is_empty());
                assert!(!toolchain.archiver_path().is_empty());
                assert_eq!(toolchain.compiler_kind(), CompilerKind::Msvc);
                assert_eq!(toolchain.archiver_kind(), ArchiverKind::Lib);
            } else {
                println!("MSVC not found on this system: {:?}", result.err());
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_detect_with_preferred_compiler() {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("cl.exe").arg("/?").output() {
                if output.status.success() {
                    let result = MsvcToolchain::detect(
                        Some("cl.exe".to_string()),
                        None,
                        None,
                        None,
                    );
                    assert!(result.is_ok());
                    
                    let toolchain = result.unwrap();
                    assert!(toolchain.compiler_path().contains("cl.exe"));
                    assert_eq!(toolchain.compiler_kind(), CompilerKind::Msvc);
                }
            }
        }
    }

    #[test]
    fn test_detect_with_preferred_linker() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(
                None,
                Some("link.exe".to_string()),
                None,
                None,
            );
            
            if let Ok(toolchain) = result {
                assert!(!toolchain.linker_path().is_empty());
                assert!(toolchain.linker_path().contains("link.exe"));
            }
        }
    }

    #[test]
    fn test_detect_with_preferred_archiver() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(
                None,
                None,
                Some("lib.exe".to_string()),
                Some(ArchiverKind::Lib),
            );
            
            if let Ok(toolchain) = result {
                assert!(toolchain.archiver_path().contains("lib.exe"));
                assert_eq!(toolchain.archiver_kind(), ArchiverKind::Lib);
            }
        }
    }

    #[test]
    fn test_toolchain_trait_methods() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(None, None, None, None);
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
                assert_eq!(_compiler_kind, CompilerKind::Msvc);
                assert_eq!(_archiver_kind, ArchiverKind::Lib);
            }
        }
    }

    #[test]
    fn test_linker_kind_is_link() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(None, None, None, None);
            if let Ok(toolchain) = result {
                assert_eq!(toolchain.linker_kind(), LinkerKind::Link);
            }
        }
    }

    #[test]
    fn test_compiler_kind_is_msvc() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(None, None, None, None);
            if let Ok(toolchain) = result {
                assert_eq!(toolchain.compiler_kind(), CompilerKind::Msvc);
            }
        }
    }

    #[test]
    fn test_system_directories() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(None, None, None, None);
            if let Ok(toolchain) = result {
                let includes = toolchain.system_include_dirs();
                let libs = toolchain.system_library_dirs();
                println!("System includes count: {}", includes.len());
                println!("System libraries count: {}", libs.len());
            }
        }
    }

    #[test]
    fn test_identify_compiler_public() {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("cl.exe").arg("/?").output() {
                if output.status.success() {
                    if let Some(kind) = MsvcToolchain::identify_compiler("cl.exe") {
                        assert_eq!(kind, CompilerKind::Msvc);
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let result = MsvcToolchain::identify_compiler("cl.exe");
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_identify_compiler_nonexistent() {
        let result = MsvcToolchain::identify_compiler("nonexistent_compiler_xyz");
        assert_eq!(result, None);
    }

    #[test]
    fn test_identify_linker_public() {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("link.exe").arg("/?").output() {
                if output.status.success() {
                    let is_linker = MsvcToolchain::identify_linker("link.exe");
                    assert!(is_linker, "link.exe should be identified as MSVC linker");
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let is_linker = MsvcToolchain::identify_linker("link.exe");
            assert!(!is_linker);
        }
    }

    #[test]
    fn test_detect_with_nonexistent_compiler() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(
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
    }

    #[test]
    fn test_detect_with_incompatible_archiver() {
        #[cfg(target_os = "windows")]
        {
            let result = MsvcToolchain::detect(
                None,
                None,
                Some("ar.exe".to_string()),
                Some(ArchiverKind::Ar),
            );
            print_result_error(&result);
        }
    }

    #[test]
    fn test_toolchain_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MsvcToolchain>();
    }

    #[test]
    fn test_consistent_detection() {
        #[cfg(target_os = "windows")]
        {
            let result1 = MsvcToolchain::detect(None, None, None, None);
            let result2 = MsvcToolchain::detect(None, None, None, None);
            
            match (&result1, &result2) {
                (Ok(tc1), Ok(tc2)) => {
                    assert_eq!(tc1.compiler_path(), tc2.compiler_path());
                    assert_eq!(tc1.linker_path(), tc2.linker_path());
                    assert_eq!(tc1.archiver_path(), tc2.archiver_path());
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
    }

    #[test]
    fn test_example_usage() {
        let toolchain = MsvcToolchain::detect(
            Some("cl.exe".to_string()),
            None,
            None,
            None,
        );

        match toolchain {
            Ok(tc) => {
                println!("Compiler: {}", tc.compiler_path());
                println!("Linker: {}", tc.linker_path());
                println!("Archiver: {}", tc.archiver_path());
            }
            Err(e) => {
                println!("Failed to detect MSVC toolchain: {}", e);
            }
        }
    }
}