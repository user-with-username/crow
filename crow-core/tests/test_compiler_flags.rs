#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use crow_core::builder::CompilerFlags;

    #[test]
    fn test_new() {
        let flags = CompilerFlags::new(CompilerKind::Gcc);
        assert_eq!(flags.get_compiler_kind(), &CompilerKind::Gcc);

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_with_compiler() {
        let flags = CompilerFlags::new(CompilerKind::Gcc).with_compiler(CompilerKind::Clang);
        assert_eq!(flags.get_compiler_kind(), &CompilerKind::Clang);
    }

    #[test]
    fn test_standard() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.standard_flags();

        let args = flags.build();
        assert_eq!(
            args,
            vec![
                "/nologo".to_string(),
                "/EHsc".to_string(),
                "/W4".to_string(),
                "/utf-8".to_string(),
            ]
        );
    }

    #[test]
    fn test_opt_level() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.optimization_level(2);

        let args = flags.build();
        assert_eq!(args, vec!["-O2".to_string()]);
    }

    #[test]
    fn test_opt_size() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.optimize_size();

        let args = flags.build();
        assert_eq!(args, vec!["-Os".to_string()]);
    }

    #[test]
    fn test_opt_speed() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.optimize_speed();

        let args = flags.build();
        assert_eq!(args, vec!["-O3".to_string()]);
    }

    #[test]
    fn test_opt_no() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.no_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["-O0".to_string()]);
    }

    #[test]
    fn test_opt_msvc() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.optimization_level(2);

        let args = flags.build();
        assert_eq!(args, vec!["/O2".to_string()]);
    }

    #[test]
    fn test_debug() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.debug_info();

        let args = flags.build();
        assert_eq!(args, vec!["-g".to_string()]);
    }

    #[test]
    fn test_debug_full() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.debug_info_full();

        let args = flags.build();
        assert_eq!(args, vec!["-g3".to_string()]);
    }

    #[test]
    fn test_no_debug() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.no_debug_info();

        let args = flags.build();
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_wall() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.all_warnings();

        let args = flags.build();
        assert_eq!(args, vec!["-Wall".to_string()]);
    }

    #[test]
    fn test_werror() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.warnings_as_errors();

        let args = flags.build();
        assert_eq!(args, vec!["-Werror".to_string()]);
    }

    #[test]
    fn test_wextra() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.extra_warnings();

        let args = flags.build();
        assert_eq!(args, vec!["-Wextra".to_string()]);
    }

    #[test]
    fn test_no_warnings() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.no_warnings();

        let args = flags.build();
        assert_eq!(args, vec!["-w".to_string()]);
    }

    #[test]
    fn test_specific_warning() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.specific_warning("unused");

        let args = flags.build();
        assert_eq!(args, vec!["-Wunused".to_string()]);
    }

    #[test]
    fn test_cxx_std() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.cxx_standard("c++17");

        let args = flags.build();
        assert_eq!(args, vec!["-std=c++17".to_string()]);
    }

    #[test]
    fn test_c_std() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.c_standard("c11");

        let args = flags.build();
        assert_eq!(args, vec!["-std=c11".to_string()]);
    }

    #[test]
    fn test_include() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.include_path("/usr/include");

        let args = flags.build();
        let norm = crow_utils::normalize_path("/usr/include");
        assert_eq!(args, vec![format!("-I{}", norm)]);
    }

    #[test]
    fn test_system_include() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.system_include_path("/usr/include");

        let args = flags.build();
        let norm = crow_utils::normalize_path("/usr/include");
        assert_eq!(args, vec!["-isystem".to_string(), norm]);
    }

    #[test]
    fn test_define() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.define("DEBUG", None::<&str>);

        let args = flags.build();
        assert_eq!(args, vec!["-DDEBUG".to_string()]);
    }

    #[test]
    fn test_define_value() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.define("VERSION", Some("1.0"));

        let args = flags.build();
        assert_eq!(args, vec!["-DVERSION=1.0".to_string()]);
    }

    #[test]
    fn test_deps_gcc() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.dependency_info("output.d");

        let args = flags.build();
        assert_eq!(
            args,
            vec!["-MD".to_string(), "-MF".to_string(), "output.d".to_string()]
        );
    }

    #[test]
    fn test_deps_msvc() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.dependency_info("output.d");

        let args = flags.build();
        assert_eq!(args, vec!["/showIncludes".to_string()]);
    }

    #[test]
    fn test_compile_only() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.compile_only();

        let args = flags.build();
        assert_eq!(args, vec!["-c".to_string()]);
    }

    #[test]
    fn test_output() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.output_file("app.exe");

        let args = flags.build();
        assert_eq!(args, vec!["/Feapp.exe".to_string()]);
    }

    #[test]
    fn test_object_output_gcc() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.object_output("main.o");

        let args = flags.build();
        assert_eq!(args, vec!["-o".to_string(), "main.o".to_string()]);
    }

    #[test]
    fn test_object_output_msvc() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.object_output("main.o");

        let args = flags.build();
        assert_eq!(args, vec!["/Fo:main.obj".to_string()]);
    }

    #[test]
    fn test_pic() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.position_independent_code();

        let args = flags.build();
        assert_eq!(args, vec!["-fPIC".to_string()]);
    }

    #[test]
    fn test_lto() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.link_time_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["-flto".to_string()]);
    }

    #[test]
    fn test_no_lto() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.no_link_time_optimization();

        let args = flags.build();
        assert_eq!(args, vec!["-fno-lto".to_string()]);
    }

    #[test]
    fn test_target_arch() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.target_arch("native");

        let args = flags.build();
        assert_eq!(args, vec!["-march=native".to_string()]);
    }

    #[test]
    fn test_nologo() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.no_logo();

        let args = flags.build();
        assert_eq!(args, vec!["/nologo".to_string()]);
    }

    #[test]
    fn test_mtdll() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.multi_threaded_dll();

        let args = flags.build();
        assert_eq!(args, vec!["/MD".to_string()]);
    }

    #[test]
    fn test_mtdll_debug() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.multi_threaded_dll_debug();

        let args = flags.build();
        assert_eq!(args, vec!["/MDd".to_string()]);
    }

    #[test]
    fn test_eh() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.exception_handling();

        let args = flags.build();
        assert_eq!(args, vec!["/EHsc".to_string()]);
    }

    #[test]
    fn test_pdb() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.program_database("output.pdb");

        let args = flags.build();
        assert_eq!(args, vec!["/Fdoutput.pdb".to_string()]);
    }

    #[test]
    fn test_debug_type() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.debug_type("Zi");

        let args = flags.build();
        assert_eq!(args, vec!["/Zi".to_string()]);
    }

    #[test]
    fn test_raw() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.add_raw("-fomit-frame-pointer");

        let args = flags.build();
        assert_eq!(args, vec!["-fomit-frame-pointer".to_string()]);
    }

    #[test]
    fn test_implib() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.import_library("libapp.a");

        let args = flags.build();
        assert_eq!(args, vec!["-Wl,--out-implib,libapp.a".to_string()]);
    }

    #[test]
    fn test_machine_gcc() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.add_machine_dependent("tune=native");

        let args = flags.build();
        assert_eq!(args, vec!["-mtune=native".to_string()]);
    }

    #[test]
    fn test_machine_msvc() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.add_machine_dependent("O2");

        let args = flags.build();
        assert_eq!(args, vec!["/O2".to_string()]);
    }

    #[test]
    fn test_feature() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.add_feature_flag("PIC");

        let args = flags.build();
        assert_eq!(args, vec!["-fPIC".to_string()]);
    }

    #[test]
    fn test_build_c_order() {
        let mut flags = CompilerFlags::new(CompilerKind::Msvc);
        flags.output_file("app.exe").compile_only();

        let args = flags.build();
        // /c must be first
        assert_eq!(args, vec!["/c".to_string(), "/Feapp.exe".to_string()]);
    }

    #[test]
    fn test_clangcl() {
        let mut flags = CompilerFlags::new(CompilerKind::ClangCl);
        flags.debug_info().optimization_level(2);

        let args = flags.build();
        assert!(args.contains(&"/O2".to_string()));
    }

    #[test]
    fn test_chain() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags
            .debug_info()
            .optimization_level(2)
            .all_warnings()
            .define("NDEBUG", None::<&str>);

        let args = flags.build();
        assert_eq!(
            args,
            vec![
                "-g".to_string(),
                "-O2".to_string(),
                "-Wall".to_string(),
                "-DNDEBUG".to_string(),
            ]
        );
    }

    #[test]
    fn test_clone() {
        let mut flags = CompilerFlags::new(CompilerKind::Gcc);
        flags.debug_info().optimization_level(2);

        let cloned = flags.clone();

        let original_args = flags.build();
        let cloned_args = cloned.build();

        assert_eq!(original_args, cloned_args);
        assert_eq!(original_args, vec!["-g".to_string(), "-O2".to_string()]);
    }
}
