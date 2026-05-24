use crate::config::macros::formatter_enum;

#[derive(Debug, Clone)]
pub enum FormatterConfig {
    Simple(String),
    Detailed {
        path: Option<String>,
        flags: Vec<String>,
        style: Option<String>,
    },
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig::Simple("clang-format".to_string())
    }
}

impl FormatterConfig {
    pub fn path(&self) -> Option<&String> {
        match self {
            FormatterConfig::Simple(path) => Some(path),
            FormatterConfig::Detailed { path, .. } => path.as_ref(),
        }
    }

    pub fn flags(&self) -> &[String] {
        match self {
            FormatterConfig::Simple(_) => &[],
            FormatterConfig::Detailed { flags, .. } => flags,
        }
    }

    pub fn style(&self) -> String {
        match self {
            FormatterConfig::Simple(style) => style.to_string(),
            FormatterConfig::Detailed { style: Some(k), .. } => k.to_string(),
            FormatterConfig::Detailed { style: None, .. } => "llvm".to_string(),
        }
    }
}

formatter_enum!(FormatterConfig);
