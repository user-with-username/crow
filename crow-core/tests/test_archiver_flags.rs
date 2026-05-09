#[cfg(test)]
mod tests {
    use crow_core::builder::flags::ArchiverFlags;
    use crow_core::builder::kinds::archiver_kind::ArchiverKind;

    #[test]
    fn test_new() {
        let flags = ArchiverFlags::new(ArchiverKind::Ar);
        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string()]);
    }

    #[test]
    fn test_output() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.output_file("output.a");

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "output.a".to_string()]);
    }

    #[test]
    fn test_output_string() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        let path = String::from("output.a");
        flags.output_file(path);

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "output.a".to_string()]);
    }

    #[test]
    fn test_obj() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.add_object("file.o");

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "file.o".to_string()]);
    }

    #[test]
    fn test_obj_string() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        let object = String::from("file.o");
        flags.add_object(object);

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "file.o".to_string()]);
    }

    #[test]
    fn test_raw() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.add_raw("--some-flag");

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "--some-flag".to_string()]);
    }

    #[test]
    fn test_raw_string() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        let raw = String::from("--some-flag");
        flags.add_raw(raw);

        let args = flags.build();
        assert_eq!(args, vec!["rcs".to_string(), "--some-flag".to_string()]);
    }

    #[test]
    fn test_ar() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags
            .output_file("lib.a")
            .add_object("main.o")
            .add_object("helper.o")
            .add_raw("--verbose");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "rcs".to_string(),
                "lib.a".to_string(),
                "main.o".to_string(),
                "helper.o".to_string(),
                "--verbose".to_string(),
            ]
        );
    }

    #[test]
    fn test_llvm_ar() {
        let mut flags = ArchiverFlags::new(ArchiverKind::LlvmAr);
        flags
            .output_file("lib.a")
            .add_object("main.o")
            .add_raw("--plugin=libLTO.so");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "rcs".to_string(),
                "lib.a".to_string(),
                "main.o".to_string(),
                "--plugin=libLTO.so".to_string(),
            ]
        );
    }

    #[test]
    fn test_lib() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Lib);
        flags
            .output_file("library.lib")
            .add_object("main.obj")
            .add_object("helper.obj")
            .add_raw("/NODEFAULTLIB");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "/OUT:library.lib".to_string(),
                "main.obj".to_string(),
                "helper.obj".to_string(),
                "/NODEFAULTLIB".to_string(),
            ]
        );
    }

    #[test]
    fn test_unknown() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Unknown);
        flags
            .output_file("output.a")
            .add_object("file.o")
            .add_raw("--custom");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "output.a".to_string(),
                "file.o".to_string(),
                "--custom".to_string(),
            ]
        );
    }

    #[test]
    fn test_empty() {
        let flags = ArchiverFlags::new(ArchiverKind::Ar);
        let args = flags.build();

        assert_eq!(args, vec!["rcs".to_string()]);
    }

    #[test]
    fn test_only_output() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.output_file("lib.a");

        let args = flags.build();

        assert_eq!(args, vec!["rcs".to_string(), "lib.a".to_string()]);
    }

    #[test]
    fn test_only_objs() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.add_object("main.o").add_object("utils.o");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "rcs".to_string(),
                "main.o".to_string(),
                "utils.o".to_string()
            ]
        );
    }

    #[test]
    fn test_lib_no_output() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Lib);
        flags.add_object("main.obj");

        let args = flags.build();

        assert_eq!(args, vec!["main.obj".to_string()]);
    }

    #[test]
    fn test_chain() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);

        flags
            .output_file("lib.a")
            .add_object("obj1.o")
            .add_raw("--flag1")
            .add_object("obj2.o")
            .add_raw("--flag2");

        let args = flags.build();
        assert_eq!(
            args,
            vec![
                "rcs".to_string(),
                "lib.a".to_string(),
                "obj1.o".to_string(),
                "--flag1".to_string(),
                "obj2.o".to_string(),
                "--flag2".to_string()
            ]
        );
    }

    #[test]
    fn test_multi_output() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.output_file("first.a").output_file("second.a");

        let args = flags.build();

        assert_eq!(
            args,
            vec![
                "rcs".to_string(),
                "first.a".to_string(),
                "second.a".to_string()
            ]
        );
    }

    #[test]
    fn test_kinds() {
        let mut ar_flags = ArchiverFlags::new(ArchiverKind::Ar);
        ar_flags.output_file("out.a");
        assert_eq!(
            ar_flags.build(),
            vec!["rcs".to_string(), "out.a".to_string()]
        );

        let mut lib_flags = ArchiverFlags::new(ArchiverKind::Lib);
        lib_flags.output_file("out.lib");
        assert_eq!(lib_flags.build(), vec!["/OUT:out.lib".to_string()]);

        let mut unknown_flags = ArchiverFlags::new(ArchiverKind::Unknown);
        unknown_flags.output_file("out.unknown");
        assert_eq!(unknown_flags.build(), vec!["out.unknown".to_string()]);
    }

    #[test]
    fn test_clone() {
        let mut flags = ArchiverFlags::new(ArchiverKind::Ar);
        flags.output_file("lib.a").add_object("main.o");

        let cloned = flags.clone();

        let original_args = flags.build();
        let cloned_args = cloned.build();

        assert_eq!(original_args, cloned_args);
        assert_eq!(
            original_args,
            vec!["rcs".to_string(), "lib.a".to_string(), "main.o".to_string()]
        );
    }
}
