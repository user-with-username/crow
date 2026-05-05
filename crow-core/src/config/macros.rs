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

#[macro_export]
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

#[macro_export]
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
                            _ => Err(E::unknown_variant(
                                value,
                                &[
                                    "bin",
                                    "exe",
                                    "lib",
                                    "static-lib",
                                    "shared-lib",
                                    "static-library",
                                    "shared-library",
                                    "module",
                                    "header-only",
                                ],
                            )),
                        }
                    }

                    fn visit_map<M>(self, map: M) -> Result<$crate::config::ProjectType, M::Error>
                    where
                        M: ::serde::de::MapAccess<'de>,
                    {
                        $crate::config::ProjectType::deserialize(
                            ::serde::de::value::MapAccessDeserializer::new(map),
                        )
                    }
                }

                deserializer.deserialize_any(ProjectTypeVisitor)
            }
        }
    };
}

#[macro_export]
macro_rules! hooks {
    ($config_type:ident, $pre_field:ident, $post_field:ident) => {
        fn deserialize_hooks<'de, D>(deserializer: D) -> Result<$config_type, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct HooksVisitor;

            impl<'de> serde::de::Visitor<'de> for HooksVisitor {
                type Value = $config_type;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(
                        "a list of strings (old format, becomes pre) or a table with pre/post arrays"
                    )
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<$config_type, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let hooks: Vec<String> = serde::Deserialize::deserialize(
                        serde::de::value::SeqAccessDeserializer::new(&mut seq)
                    )?;
                    Ok($config_type {
                        $pre_field: hooks,
                        $post_field: Vec::new(),
                    })
                }

                fn visit_map<M>(self, mut map: M) -> Result<$config_type, M::Error>
                where
                    M: serde::de::MapAccess<'de>,
                {
                    let mut pre: Vec<String> = Vec::new();
                    let mut post: Vec<String> = Vec::new();

                    while let Some(key) = map.next_key::<String>()? {
                        match key.as_str() {
                            stringify!($pre_field) => pre = map.next_value()?,
                            stringify!($post_field) => post = map.next_value()?,
                            _ => {
                                return Err(serde::de::Error::unknown_field(
                                    &key,
                                    &[stringify!($pre_field), stringify!($post_field)],
                                ));
                            }
                        }
                    }

                    Ok($config_type {
                        $pre_field: pre,
                        $post_field: post,
                    })
                }
            }

            deserializer.deserialize_any(HooksVisitor)
        }

        impl<'de> serde::Deserialize<'de> for $config_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserialize_hooks(deserializer)
            }
        }
    };
}

pub(crate) use config_enum;
pub(crate) use hooks;
pub(crate) use type_enum;
