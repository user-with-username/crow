#![allow(unused_imports)]

use serde::de::{self, Deserializer, MapAccess};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct DetailedConfig<T> {
    pub(crate) path: Option<String>,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
    pub(crate) kind: Option<T>,
}

macro_rules! config_enum {
    ($name:ident, $kind:ty) => {
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct ConfigVisitor;

                impl<'de> ::serde::de::Visitor<'de> for ConfigVisitor {
                    type Value = $name;

                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        formatter.write_str("a string (path) or a struct with path, flags, and kind")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        let kind = match <$kind>::deserialize(
                            ::serde::de::value::StrDeserializer::<::serde::de::value::Error>::new(value)
                        ) {
                            Ok(k) => Some(k),
                            Err(_) => None,
                        };

                        Ok($name::Detailed {
                            path: Some(value.to_string()),
                            flags: Vec::new(),
                            kind,
                        })
                    }

                    fn visit_map<M>(self, map: M) -> Result<$name, M::Error>
                    where
                        M: ::serde::de::MapAccess<'de>,
                    {
                        let detailed =
                            $crate::config::macros::DetailedConfig::<$kind>::deserialize(
                                ::serde::de::value::MapAccessDeserializer::new(map),
                            )?;
                        Ok($name::Detailed {
                            path: detailed.path,
                            flags: detailed.flags,
                            kind: detailed.kind,
                        })
                    }
                }

                deserializer.deserialize_any(ConfigVisitor)
            }
        }
    };
}

pub(crate) use config_enum;
