#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use crow_core::builder::LinkerFlags;

    #[test]
    fn test_new() {
        let flags = LinkerFlags::new(CompilerKind::Gcc);
        assert_eq!(flags.get_compiler_kind(), &CompilerKind::Gcc);

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_with_compiler() {
        let flags = LinkerFlags::new(CompilerKind::Gcc).with_compiler(CompilerKind::Clang);
        assert_eq!(flags.get_compiler_kind(), &CompilerKind::Clang);
    }

    #[test]
    fn test_lld() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.use_lld();

        let args = flags.build();
        assert_eq!(args, vec!["-fuse-ld=lld".to_string()]);
    }

    #[test]
    fn test_lld_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.use_lld();

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_standard_flags() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.standard_flags();

        let args = flags.build();
        assert_eq!(args, vec!["/nologo".to_string()]);
    }

    #[test]
    fn test_debug() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.debug_info();

        let args = flags.build();
        assert_eq!(args, vec!["-g".to_string()]);
    }

    #[test]
    fn test_debug_full() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.debug_info_full();

        let args = flags.build();
        assert_eq!(args, vec!["-g3".to_string()]);
    }

    #[test]
    fn test_no_debug() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.no_debug_info();

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_output() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.output_file("output.exe");

        let args = flags.build();
        assert_eq!(args, vec!["-o".to_string(), "output.exe".to_string()]);
    }

    #[test]
    fn test_output_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.output_file("output.exe");

        let args = flags.build();
        assert_eq!(args, vec!["/OUT:output.exe".to_string()]);
    }

    #[test]
    fn test_link_lib() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.link_library("mylib");

        let args = flags.build();
        assert_eq!(args, vec!["-lmylib".to_string()]);
    }

    #[test]
    fn test_link_lib_path() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.link_library("libmylib.a");

        let args = flags.build();
        assert_eq!(args, vec!["libmylib.a".to_string()]);
    }

    #[test]
    fn test_link_lib_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.link_library("mylib");

        let args = flags.build();
        assert_eq!(args, vec!["mylib.lib".to_string()]);
    }

    #[test]
    fn test_lib_path() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.library_path("/usr/lib");

        let args = flags.build();
        let norm = crow_utils::normalize_path("/usr/lib");
        assert_eq!(args, vec![format!("-L{}", norm)]);
    }

    #[test]
    fn test_lib_path_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.library_path("C:\\libs");

        let args = flags.build();
        let norm = crow_utils::normalize_path("C:\\libs");
        assert_eq!(args, vec![format!("/LIBPATH:{}", norm)]);
    }

    #[test]
    fn test_static() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.static_link();

        let args = flags.build();
        assert_eq!(args, vec!["-static".to_string()]);
    }

    #[test]
    fn test_shared() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.shared_link();

        let args = flags.build();
        assert_eq!(args, vec!["-shared".to_string()]);
    }

    #[test]
    fn test_lto() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.link_time_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["-flto".to_string()]);
    }

    #[test]
    fn test_lto_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.link_time_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["/LTCG".to_string()]);
    }

    #[test]
    fn test_no_lto() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.no_link_time_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["-fno-lto".to_string()]);
    }

    #[test]
    fn test_thin_lto() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.thin_lto();

        let args = flags.build();
        assert_eq!(args, vec!["-flto=thin".to_string()]);
    }

    #[test]
    fn test_fat_lto() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.fat_lto();

        let args = flags.build();
        assert_eq!(args, vec!["-flto=full".to_string()]);
    }

    #[test]
    fn test_arch() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.target_arch("native");

        let args = flags.build();
        assert_eq!(args, vec!["-march=native".to_string()]);
    }

    #[test]
    fn test_arch_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.target_arch("AVX2");

        let args = flags.build();
        assert_eq!(args, vec!["/arch:AVX2".to_string()]);
    }

    #[test]
    fn test_nologo() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.no_logo();

        let args = flags.build();
        assert_eq!(args, vec!["/nologo".to_string()]);
    }

    #[test]
    fn test_pdb() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.link_program_database("output.pdb");

        let args = flags.build();
        assert_eq!(args, vec!["/PDB:output.pdb".to_string()]);
    }

    #[test]
    fn test_raw() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.add_raw("--custom-flag");

        let args = flags.build();
        assert_eq!(args, vec!["--custom-flag".to_string()]);
    }

    #[test]
    fn test_machine() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.add_machine_dependent("tune=native");

        let args = flags.build();
        assert_eq!(args, vec!["-mtune=native".to_string()]);
    }

    #[test]
    fn test_machine_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.add_machine_dependent("OPT:REF");

        let args = flags.build();
        assert_eq!(args, vec!["/OPT:REF".to_string()]);
    }

    #[test]
    fn test_shared_lib_gcc() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.shared_library();

        let args = flags.build();
        assert_eq!(args, vec!["-shared".to_string()]);
    }

    #[test]
    fn test_shared_lib_msvc() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.shared_library();

        let args = flags.build();
        assert_eq!(args, vec!["/LD".to_string()]);
    }

    #[test]
    fn test_implib() {
        let mut flags = LinkerFlags::new(CompilerKind::Msvc);
        flags.import_library("output.lib");

        let args = flags.build();
        assert_eq!(args, vec!["/IMPLIB:output.lib".to_string()]);
    }

    #[test]
    fn test_implib_gcc() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.import_library("output.lib");

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_chain() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags
            .debug_info()
            .output_file("app")
            .link_library("m")
            .static_link();

        let args = flags.build();
        assert_eq!(
            args,
            vec![
                "-g".to_string(),
                "-o".to_string(),
                "app".to_string(),
                "-lm".to_string(),
                "-static".to_string(),
            ]
        );
    }

    #[test]
    fn test_clangcl() {
        let mut flags = LinkerFlags::new(CompilerKind::ClangCl);
        flags
            .debug_info()
            .output_file("app.exe")
            .link_library("user32");

        let args = flags.build();
        // ClangCl uses MSVC style flags
        assert_eq!(
            args,
            vec![
                "/DEBUG".to_string(),
                "/OUT:app.exe".to_string(),
                "user32.lib".to_string(),
            ]
        );
    }

    #[test]
    fn test_clone() {
        let mut flags = LinkerFlags::new(CompilerKind::Gcc);
        flags.debug_info().output_file("app");

        let cloned = flags.clone();

        let original_args = flags.build();
        let cloned_args = cloned.build();

        assert_eq!(original_args, cloned_args);
        assert_eq!(
            original_args,
            vec!["-g".to_string(), "-o".to_string(), "app".to_string()]
        );
    }
}
