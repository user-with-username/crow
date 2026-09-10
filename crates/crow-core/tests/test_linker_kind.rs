#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::linker_kind::LinkerKind;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Wrapper {
        kind: LinkerKind,
    }

    #[test]
    fn test_serialize() {
        let w = Wrapper {
            kind: LinkerKind::Ld,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"ld\"\n");

        let w = Wrapper {
            kind: LinkerKind::AppleLd,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"apple_ld\"\n");

        let w = Wrapper {
            kind: LinkerKind::WasmLd,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"wasm_ld\"\n");

        let w = Wrapper {
            kind: LinkerKind::Unknown,
        };
        assert_eq!(toml::to_string(&w).unwrap(), "kind = \"unknown\"\n");
    }

    #[test]
    fn test_deserialize() {
        let w: Wrapper = toml::from_str("kind = \"ld\"").unwrap();
        assert_eq!(w.kind, LinkerKind::Ld);

        let w: Wrapper = toml::from_str("kind = \"lld\"").unwrap();
        assert_eq!(w.kind, LinkerKind::Lld);

        let w: Wrapper = toml::from_str("kind = \"apple_ld\"").unwrap();
        assert_eq!(w.kind, LinkerKind::AppleLd);

        let w: Wrapper = toml::from_str("kind = \"wasm_ld\"").unwrap();
        assert_eq!(w.kind, LinkerKind::WasmLd);

        let w: Wrapper = toml::from_str("kind = \"gold\"").unwrap();
        assert_eq!(w.kind, LinkerKind::Gold);

        let w: Wrapper = toml::from_str("kind = \"unknown\"").unwrap();
        assert_eq!(w.kind, LinkerKind::Unknown);
    }

    #[test]
    fn test_roundtrip() {
        let kinds = [
            LinkerKind::Ld,
            LinkerKind::Lld,
            LinkerKind::Link,
            LinkerKind::AppleLd,
            LinkerKind::WasmLd,
            LinkerKind::Gold,
            LinkerKind::BpfLink,
            LinkerKind::Unknown,
        ];

        for kind in kinds {
            let w = Wrapper { kind };
            let toml_str = toml::to_string(&w).unwrap();
            let decoded: Wrapper = toml::from_str(&toml_str).unwrap();
            assert_eq!(w.kind, decoded.kind);
        }
    }
}
