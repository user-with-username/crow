#[cfg(test)]
mod tests {
    use crow_core::builder::flags::flag::ArchiverFlag;
    use crow_core::builder::flags::flag::Flag;

    #[test]
    fn test_optimization_level() {
        let flag = Flag::OptimizationLevel(3);
        match flag {
            Flag::OptimizationLevel(level) => assert_eq!(level, 3),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_optimize_size() {
        let flag = Flag::OptimizeSize;
        assert!(matches!(flag, Flag::OptimizeSize));
    }

    #[test]
    fn test_optimize_speed() {
        let flag = Flag::OptimizeSpeed;
        assert!(matches!(flag, Flag::OptimizeSpeed));
    }

    #[test]
    fn test_no_optimization() {
        let flag = Flag::NoOptimization;
        assert!(matches!(flag, Flag::NoOptimization));
    }

    #[test]
    fn test_dependency_info() {
        let flag = Flag::DependencyInfo("output.d".to_string());
        match flag {
            Flag::DependencyInfo(path) => assert_eq!(path, "output.d"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_debug_info() {
        let flag = Flag::DebugInfo;
        assert!(matches!(flag, Flag::DebugInfo));
    }

    #[test]
    fn test_debug_info_full() {
        let flag = Flag::DebugInfoFull;
        assert!(matches!(flag, Flag::DebugInfoFull));
    }

    #[test]
    fn test_no_debug_info() {
        let flag = Flag::NoDebugInfo;
        assert!(matches!(flag, Flag::NoDebugInfo));
    }

    #[test]
    fn test_all_warnings() {
        let flag = Flag::AllWarnings;
        assert!(matches!(flag, Flag::AllWarnings));
    }

    #[test]
    fn test_warnings_as_errors() {
        let flag = Flag::WarningsAsErrors;
        assert!(matches!(flag, Flag::WarningsAsErrors));
    }

    #[test]
    fn test_extra_warnings() {
        let flag = Flag::ExtraWarnings;
        assert!(matches!(flag, Flag::ExtraWarnings));
    }

    #[test]
    fn test_no_warnings() {
        let flag = Flag::NoWarnings;
        assert!(matches!(flag, Flag::NoWarnings));
    }

    #[test]
    fn test_specific_warning() {
        let flag = Flag::SpecificWarning("unused".to_string());
        match flag {
            Flag::SpecificWarning(warning) => assert_eq!(warning, "unused"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_cxx_standard() {
        let flag = Flag::CxxStandard("c++17".to_string());
        match flag {
            Flag::CxxStandard(std) => assert_eq!(std, "c++17"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_c_standard() {
        let flag = Flag::CStandard("c11".to_string());
        match flag {
            Flag::CStandard(std) => assert_eq!(std, "c11"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_include_path() {
        let flag = Flag::IncludePath("/usr/include".to_string());
        match flag {
            Flag::IncludePath(path) => assert_eq!(path, "/usr/include"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_system_include_path() {
        let flag = Flag::SystemIncludePath("/usr/include".to_string());
        match flag {
            Flag::SystemIncludePath(path) => assert_eq!(path, "/usr/include"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_define_without_value() {
        let flag = Flag::Define("DEBUG".to_string(), None);
        match flag {
            Flag::Define(name, None) => assert_eq!(name, "DEBUG"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_define_with_value() {
        let flag = Flag::Define("VERSION".to_string(), Some("1.0".to_string()));
        match flag {
            Flag::Define(name, Some(value)) => {
                assert_eq!(name, "VERSION");
                assert_eq!(value, "1.0");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_compile_only() {
        let flag = Flag::CompileOnly;
        assert!(matches!(flag, Flag::CompileOnly));
    }

    #[test]
    fn test_output_file() {
        let flag = Flag::OutputFile("app.exe".to_string());
        match flag {
            Flag::OutputFile(file) => assert_eq!(file, "app.exe"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_object_output() {
        let flag = Flag::ObjectOutput("main.o".to_string());
        match flag {
            Flag::ObjectOutput(obj) => assert_eq!(obj, "main.o"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_link_library() {
        let flag = Flag::LinkLibrary("mylib".to_string());
        match flag {
            Flag::LinkLibrary(lib) => assert_eq!(lib, "mylib"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_library_path() {
        let flag = Flag::LibraryPath("/usr/lib".to_string());
        match flag {
            Flag::LibraryPath(path) => assert_eq!(path, "/usr/lib"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_static_link() {
        let flag = Flag::StaticLink;
        assert!(matches!(flag, Flag::StaticLink));
    }

    #[test]
    fn test_shared_link() {
        let flag = Flag::SharedLink;
        assert!(matches!(flag, Flag::SharedLink));
    }

    #[test]
    fn test_position_independent_code() {
        let flag = Flag::PositionIndependentCode;
        assert!(matches!(flag, Flag::PositionIndependentCode));
    }

    #[test]
    fn test_link_time_optimization() {
        let flag = Flag::LinkTimeOptimization;
        assert!(matches!(flag, Flag::LinkTimeOptimization));
    }

    #[test]
    fn test_no_link_time_optimization() {
        let flag = Flag::NoLinkTimeOptimization;
        assert!(matches!(flag, Flag::NoLinkTimeOptimization));
    }

    #[test]
    fn test_thin_lto() {
        let flag = Flag::ThinLTO;
        assert!(matches!(flag, Flag::ThinLTO));
    }

    #[test]
    fn test_fat_lto() {
        let flag = Flag::FatLTO;
        assert!(matches!(flag, Flag::FatLTO));
    }

    #[test]
    fn test_target_arch() {
        let flag = Flag::TargetArch("native".to_string());
        match flag {
            Flag::TargetArch(arch) => assert_eq!(arch, "native"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_no_logo() {
        let flag = Flag::NoLogo;
        assert!(matches!(flag, Flag::NoLogo));
    }

    #[test]
    fn test_multi_threaded_dll() {
        let flag = Flag::MultiThreadedDLL;
        assert!(matches!(flag, Flag::MultiThreadedDLL));
    }

    #[test]
    fn test_multi_threaded_dll_debug() {
        let flag = Flag::MultiThreadedDLLDebug;
        assert!(matches!(flag, Flag::MultiThreadedDLLDebug));
    }

    #[test]
    fn test_exception_handling() {
        let flag = Flag::ExceptionHandling;
        assert!(matches!(flag, Flag::ExceptionHandling));
    }

    #[test]
    fn test_machine_dependent() {
        let flag = Flag::MachineDependent("tune=native".to_string());
        match flag {
            Flag::MachineDependent(opt) => assert_eq!(opt, "tune=native"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_feature_flag() {
        let flag = Flag::FeatureFlag("PIC".to_string());
        match flag {
            Flag::FeatureFlag(feature) => assert_eq!(feature, "PIC"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_show_includes() {
        let flag = Flag::ShowIncludes;
        assert!(matches!(flag, Flag::ShowIncludes));
    }

    #[test]
    fn test_source_dependencies() {
        let flag = Flag::SourceDependencies("deps.d".to_string());
        match flag {
            Flag::SourceDependencies(file) => assert_eq!(file, "deps.d"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_program_database() {
        let flag = Flag::ProgramDatabase("output.pdb".to_string());
        match flag {
            Flag::ProgramDatabase(pdb) => assert_eq!(pdb, "output.pdb"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_link_program_database() {
        let flag = Flag::LinkProgramDatabase("output.pdb".to_string());
        match flag {
            Flag::LinkProgramDatabase(pdb) => assert_eq!(pdb, "output.pdb"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_debug_type() {
        let flag = Flag::DebugType("Zi".to_string());
        match flag {
            Flag::DebugType(debug_type) => assert_eq!(debug_type, "Zi"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_import_library() {
        let flag = Flag::ImportLibrary("libapp.a".to_string());
        match flag {
            Flag::ImportLibrary(lib) => assert_eq!(lib, "libapp.a"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_raw() {
        let flag = Flag::Raw("-fomit-frame-pointer".to_string());
        match flag {
            Flag::Raw(flag_str) => assert_eq!(flag_str, "-fomit-frame-pointer"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_archiver_output_file() {
        let flag = ArchiverFlag::OutputFile("libarchive.a".to_string());
        match flag {
            ArchiverFlag::OutputFile(file) => assert_eq!(file, "libarchive.a"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_archiver_object() {
        let flag = ArchiverFlag::Object("file.o".to_string());
        match flag {
            ArchiverFlag::Object(obj) => assert_eq!(obj, "file.o"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_archiver_raw() {
        let flag = ArchiverFlag::Raw("-s".to_string());
        match flag {
            ArchiverFlag::Raw(flag_str) => assert_eq!(flag_str, "-s"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_flag_clone() {
        let flag = Flag::OptimizationLevel(2);
        let cloned = flag.clone();

        match (flag, cloned) {
            (Flag::OptimizationLevel(orig), Flag::OptimizationLevel(cloned)) => {
                assert_eq!(orig, cloned);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_archiver_flag_clone() {
        let flag = ArchiverFlag::OutputFile("lib.a".to_string());
        let cloned = flag.clone();

        match (flag, cloned) {
            (ArchiverFlag::OutputFile(orig), ArchiverFlag::OutputFile(cloned)) => {
                assert_eq!(orig, cloned);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_flag_debug_formatting() {
        let flag = Flag::Define("TEST".to_string(), Some("1".to_string()));
        let debug_str = format!("{:?}", flag);
        assert!(debug_str.contains("Define"));
        assert!(debug_str.contains("TEST"));
    }

    #[test]
    fn test_archiver_flag_debug_formatting() {
        let flag = ArchiverFlag::Object("main.o".to_string());
        let debug_str = format!("{:?}", flag);
        assert!(debug_str.contains("Object"));
        assert!(debug_str.contains("main.o"));
    }

    #[test]
    fn test_flag_equality() {
        let flag1 = Flag::OptimizationLevel(3);
        let flag2 = Flag::OptimizationLevel(3);
        let flag3 = Flag::OptimizationLevel(2);

        assert!(
            matches!((&flag1, &flag2), (Flag::OptimizationLevel(a), Flag::OptimizationLevel(b)) if a == b)
        );
        assert!(
            !matches!((&flag1, &flag3), (Flag::OptimizationLevel(a), Flag::OptimizationLevel(b)) if a == b)
        );
    }

    #[test]
    fn test_archiver_flag_equality() {
        let flag1 = ArchiverFlag::OutputFile("lib.a".to_string());
        let flag2 = ArchiverFlag::OutputFile("lib.a".to_string());

        assert!(
            matches!((&flag1, &flag2), (ArchiverFlag::OutputFile(a), ArchiverFlag::OutputFile(b)) if a == b)
        );
    }

    #[test]
    fn test_flag_vec_collection() {
        let mut flags: Vec<Flag> = Vec::new();
        flags.push(Flag::DebugInfo);
        flags.push(Flag::OptimizationLevel(2));
        flags.push(Flag::AllWarnings);

        assert_eq!(flags.len(), 3);
        assert!(matches!(flags[0], Flag::DebugInfo));
        assert!(matches!(flags[1], Flag::OptimizationLevel(2)));
        assert!(matches!(flags[2], Flag::AllWarnings));
    }

    #[test]
    fn test_archiver_flag_vec_collection() {
        let mut flags: Vec<ArchiverFlag> = Vec::new();
        flags.push(ArchiverFlag::OutputFile("lib.a".to_string()));
        flags.push(ArchiverFlag::Object("file.o".to_string()));

        assert_eq!(flags.len(), 2);
        assert!(matches!(&flags[0], ArchiverFlag::OutputFile(_)));
        assert!(matches!(&flags[1], ArchiverFlag::Object(_)));
    }

    #[test]
    fn test_optimization_pattern_matching() {
        let flags = vec![
            Flag::OptimizationLevel(1),
            Flag::OptimizeSize,
            Flag::OptimizeSpeed,
            Flag::NoOptimization,
        ];

        let mut opt_levels = 0;
        let mut has_size = false;
        let mut has_speed = false;
        let mut has_no = false;

        for flag in flags {
            match flag {
                Flag::OptimizationLevel(_) => opt_levels += 1,
                Flag::OptimizeSize => has_size = true,
                Flag::OptimizeSpeed => has_speed = true,
                Flag::NoOptimization => has_no = true,
                _ => (),
            }
        }

        assert_eq!(opt_levels, 1);
        assert!(has_size);
        assert!(has_speed);
        assert!(has_no);
    }

    #[test]
    fn test_debug_pattern_matching() {
        let flags = vec![Flag::DebugInfo, Flag::DebugInfoFull, Flag::NoDebugInfo];

        let mut has_debug = false;
        let mut has_debug_full = false;
        let mut has_no_debug = false;

        for flag in flags {
            match flag {
                Flag::DebugInfo => has_debug = true,
                Flag::DebugInfoFull => has_debug_full = true,
                Flag::NoDebugInfo => has_no_debug = true,
                _ => (),
            }
        }

        assert!(has_debug);
        assert!(has_debug_full);
        assert!(has_no_debug);
    }
}
