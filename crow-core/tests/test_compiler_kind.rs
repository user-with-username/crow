#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use serde::{Deserialize, Serialize};
    use toml;

    #[derive(Serialize, Deserialize)]
    struct Wrapper {
        kind: CompilerKind,
    }

    #[test]
    fn test_serialize() {
        let w = Wrapper {
            kind: CompilerKind::Gcc,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"gcc\"\n");

        let w = Wrapper {
            kind: CompilerKind::Gpp,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"gpp\"\n");

        let w = Wrapper {
            kind: CompilerKind::Clang,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"clang\"\n");

        let w = Wrapper {
            kind: CompilerKind::ClangPP,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"clang_p_p\"\n");

        let w = Wrapper {
            kind: CompilerKind::ClangCl,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"clang_cl\"\n");

        let w = Wrapper {
            kind: CompilerKind::Msvc,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"msvc\"\n");

        let w = Wrapper {
            kind: CompilerKind::Unknown,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"unknown\"\n");
    }

    #[test]
    fn test_deserialize() {
        let w: Wrapper = toml::from_str("kind = \"gcc\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Gcc);

        let w: Wrapper = toml::from_str("kind = \"gpp\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Gpp);

        let w: Wrapper = toml::from_str("kind = \"g++\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Gpp);

        let w: Wrapper = toml::from_str("kind = \"clang\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Clang);

        let w: Wrapper = toml::from_str("kind = \"clang_p_p\"").unwrap();
        assert_eq!(w.kind, CompilerKind::ClangPP);

        let w: Wrapper = toml::from_str("kind = \"clang++\"").unwrap();
        assert_eq!(w.kind, CompilerKind::ClangPP);

        let w: Wrapper = toml::from_str("kind = \"clang_cl\"").unwrap();
        assert_eq!(w.kind, CompilerKind::ClangCl);

        let w: Wrapper = toml::from_str("kind = \"msvc\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Msvc);

        let w: Wrapper = toml::from_str("kind = \"unknown\"").unwrap();
        assert_eq!(w.kind, CompilerKind::Unknown);
    }

    #[test]
    fn test_roundtrip() {
        let kinds = [
            CompilerKind::Gcc,
            CompilerKind::Gpp,
            CompilerKind::Clang,
            CompilerKind::ClangPP,
            CompilerKind::ClangCl,
            CompilerKind::Msvc,
            CompilerKind::Unknown,
        ];

        for kind in kinds {
            let w = Wrapper { kind };
            let toml_str = toml::to_string(&w).unwrap();
            let decoded: Wrapper = toml::from_str(&toml_str).unwrap();
            assert_eq!(w.kind, decoded.kind);
        }
    }

    #[test]
    fn test_is_msvc() {
        assert!(CompilerKind::Msvc.is_msvc());
        assert!(CompilerKind::ClangCl.is_msvc());

        assert!(!CompilerKind::Gcc.is_msvc());
        assert!(!CompilerKind::Gpp.is_msvc());
        assert!(!CompilerKind::Clang.is_msvc());
        assert!(!CompilerKind::ClangPP.is_msvc());
        assert!(!CompilerKind::Unknown.is_msvc());
    }

    #[test]
    fn test_is_gnu() {
        assert!(CompilerKind::Gcc.is_gnu());
        assert!(CompilerKind::Gpp.is_gnu());
        assert!(CompilerKind::Clang.is_gnu());
        assert!(CompilerKind::ClangPP.is_gnu());

        assert!(!CompilerKind::ClangCl.is_gnu());
        assert!(!CompilerKind::Msvc.is_gnu());
        assert!(!CompilerKind::Unknown.is_gnu());
    }
}
