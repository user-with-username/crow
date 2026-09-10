#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::archiver_kind::ArchiverKind;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Wrapper {
        kind: ArchiverKind,
    }

    #[test]
    fn test_serialize() {
        let w = Wrapper {
            kind: ArchiverKind::Ar,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"ar\"\n");

        let w = Wrapper {
            kind: ArchiverKind::Lib,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"lib\"\n");

        let w = Wrapper {
            kind: ArchiverKind::LlvmAr,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"llvm_ar\"\n");

        let w = Wrapper {
            kind: ArchiverKind::Unknown,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"unknown\"\n");
    }

    #[test]
    fn test_deserialize() {
        let w: Wrapper = toml::from_str("kind = \"ar\"").unwrap();
        assert_eq!(w.kind, ArchiverKind::Ar);

        let w: Wrapper = toml::from_str("kind = \"lib\"").unwrap();
        assert_eq!(w.kind, ArchiverKind::Lib);

        let w: Wrapper = toml::from_str("kind = \"llvm_ar\"").unwrap();
        assert_eq!(w.kind, ArchiverKind::LlvmAr);

        let w: Wrapper = toml::from_str("kind = \"unknown\"").unwrap();
        assert_eq!(w.kind, ArchiverKind::Unknown);
    }

    #[test]
    fn test_roundtrip() {
        let kinds = [
            ArchiverKind::Ar,
            ArchiverKind::Lib,
            ArchiverKind::LlvmAr,
            ArchiverKind::Unknown,
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
        assert!(ArchiverKind::Lib.is_msvc());

        assert!(!ArchiverKind::Ar.is_msvc());
        assert!(!ArchiverKind::LlvmAr.is_msvc());
        assert!(!ArchiverKind::Unknown.is_msvc());
    }

    #[test]
    fn test_is_gnu() {
        assert!(ArchiverKind::Ar.is_gnu());
        assert!(ArchiverKind::LlvmAr.is_gnu());

        assert!(!ArchiverKind::Lib.is_gnu());
        assert!(!ArchiverKind::Unknown.is_gnu());
    }
}
