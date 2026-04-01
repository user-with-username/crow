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
                        formatter
                            .write_str("a string (path) or a struct with path, flags, and kind")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        let kind =
                            match <$kind>::deserialize(::serde::de::value::StrDeserializer::<
                                ::serde::de::value::Error,
                            >::new(value))
                            {
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

macro_rules! type_enum {
    () => {
        impl<'de> ::serde::Deserialize<'de> for $crate::config::ProjectType {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct ProjectTypeVisitor;

                impl<'de> ::serde::de::Visitor<'de> for ProjectTypeVisitor {
                    type Value = $crate::config::ProjectType;

                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        formatter.write_str("a string (type name) or a table with type field")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$crate::config::ProjectType, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            "bin" => Ok($crate::config::ProjectType::Bin(Default::default())),
                            "exe" => Ok($crate::config::ProjectType::Exe(Default::default())),
                            "lib" => Ok($crate::config::ProjectType::Lib(Default::default())),
                            "static-lib" | "staticlib" => {
                                Ok($crate::config::ProjectType::StaticLib(Default::default()))
                            }
                            "shared-lib" | "sharedlib" => {
                                Ok($crate::config::ProjectType::SharedLib(Default::default()))
                            }
                            "module" => Ok($crate::config::ProjectType::Module(Default::default())),
                            "header-only" => Ok($crate::config::ProjectType::HeaderOnly),
                            _ => Err(E::unknown_variant(value, &[
                                "bin", "exe", "lib", "static-lib", "shared-lib",
                                "static-library", "shared-library", "module", "header-only"
                            ])),
                        }
                    }

                    fn visit_map<M>(self, map: M) -> Result<$crate::config::ProjectType, M::Error>
                    where
                        M: ::serde::de::MapAccess<'de>,
                    {
                        $crate::config::ProjectType::deserialize(
                            ::serde::de::value::MapAccessDeserializer::new(map)
                        )
                    }
                }

                deserializer.deserialize_any(ProjectTypeVisitor)
            }
        }
    };
}

pub(crate) use type_enum;
pub(crate) use config_enum;
