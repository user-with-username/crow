use crate::config::ArchiverConfig;
use crate::config::CompilerConfig;
use crate::config::FormatterConfig;
use crate::config::LinkerConfig;
use crow_utils::condition::CfgEvaluator;
use serde::Deserialize;
use serde::Deserializer;
use smart_default::SmartDefault;
use std::path::PathBuf;

#[derive(Debug, Deserialize, SmartDefault, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub compiler: CompilerConfig,
    pub linker: LinkerConfig,
    pub archiver: ArchiverConfig,
    pub formatter: FormatterConfig,

    #[default(vec![PathBuf::from("include")])]
    pub include_dirs: Vec<PathBuf>,

    #[default(Vec::new())]
    pub lib_dirs: Vec<PathBuf>,

    #[default(Vec::new())]
    pub libs: Vec<String>,

    #[default(vec![PathBuf::from("src")])]
    pub src_dirs: Vec<PathBuf>,

    #[default(vec![PathBuf::from("tests")])]
    pub test_dirs: Vec<PathBuf>,

    #[default(vec![PathBuf::from("benches")])]
    pub bench_dirs: Vec<PathBuf>,

    #[default(vec!["cpp".to_string(), "c".to_string(), "cc".to_string(), "cxx".to_string()])]
    pub src_extensions: Vec<String>,

    #[default(Vec::new())]
    pub preprocessor_defines: Vec<String>,

    #[serde(default)]
    pub hooks: HooksConfig,

    #[default(false)]
    pub warnings_as_errors: bool,

    #[default(true)]
    pub parallelism: bool,
}

#[derive(Debug, SmartDefault, Clone)]
pub struct HooksConfig {
    #[default(Vec::new())]
    pub pre: Vec<String>,

    #[default(Vec::new())]
    pub post: Vec<String>,
}

impl<'de> Deserialize<'de> for HooksConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HooksVisitor;

        impl<'de> serde::de::Visitor<'de> for HooksVisitor {
            type Value = HooksConfig;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(
                    "a list of hooks (old format, becomes pre) or a table with pre/post arrays",
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let hooks: Vec<toml::Value> = Deserialize::deserialize(
                    serde::de::value::SeqAccessDeserializer::new(&mut seq),
                )?;
                let pre = filter_conditional_hooks(hooks).map_err(serde::de::Error::custom)?;
                Ok(HooksConfig {
                    pre,
                    post: Vec::new(),
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut pre = Vec::new();
                let mut post = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "pre" => {
                            let hooks: Vec<toml::Value> = map.next_value()?;
                            pre = filter_conditional_hooks(hooks)
                                .map_err(serde::de::Error::custom)?;
                        }
                        "post" => {
                            let hooks: Vec<toml::Value> = map.next_value()?;
                            post = filter_conditional_hooks(hooks)
                                .map_err(serde::de::Error::custom)?;
                        }
                        other => {
                            return Err(serde::de::Error::unknown_field(other, &["pre", "post"]));
                        }
                    }
                }
                Ok(HooksConfig { pre, post })
            }
        }

        deserializer.deserialize_any(HooksVisitor)
    }
}

/// Filter a list of hooks, keeping only entries whose `target` condition matches.
fn filter_conditional_hooks(hooks: Vec<toml::Value>) -> Result<Vec<String>, String> {
    let evaluator = CfgEvaluator::new();
    let mut result = Vec::new();
    for item in hooks {
        match item {
            toml::Value::String(s) => result.push(s),
            toml::Value::Table(mut tab) => {
                let cmd = tab
                    .remove("cmd")
                    .and_then(|v| v.as_str().map(String::from))
                    .ok_or("conditional hook table missing 'cmd' field")?;
                if let Some(target_val) = tab.remove("target") {
                    let condition = target_val
                        .as_str()
                        .ok_or_else(|| "target must be a string".to_string())?;
                    if evaluator.eval(condition) {
                        result.push(cmd);
                    }
                } else {
                    result.push(cmd);
                }
            }
            _ => return Err("expected string or table for hook".to_string()),
        }
    }
    Ok(result)
}
