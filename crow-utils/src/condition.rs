use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct CfgEvaluator {
    values: BTreeMap<String, Value>,
}

impl CfgEvaluator {
    pub fn new() -> Self {
        let mut values = BTreeMap::new();

        values.insert("windows".into(), Value::Bool(cfg!(target_os = "windows")));
        values.insert("unix".into(), Value::Bool(cfg!(unix)));
        values.insert(
            "debug_assertions".into(),
            Value::Bool(cfg!(debug_assertions)),
        );

        // target_os
        values.insert(
            "target_os".into(),
            Value::Str(Self::target_os_str().to_string()),
        );

        // target_arch
        values.insert(
            "target_arch".into(),
            Value::Str(Self::target_arch_str().to_string()),
        );

        // target_family
        values.insert(
            "target_family".into(),
            Value::Str(Self::target_family_str().to_string()),
        );

        // target_env
        values.insert(
            "target_env".into(),
            Value::Str(Self::target_env_str().to_string()),
        );

        // target_pointer_width
        values.insert(
            "target_pointer_width".into(),
            Value::Str(Self::target_pointer_width_str().to_string()),
        );

        // target_endian
        values.insert(
            "target_endian".into(),
            Value::Str(if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            }
            .to_string()),
        );

        Self { values }
    }

    pub fn with_values(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    pub fn eval(&self, expr: &str) -> bool {
        let input = expr.trim();
        if input.is_empty() {
            return true;
        }
        match self.parse_or(input) {
            Ok((rest, result)) => rest.trim().is_empty() && result,
            Err(_) => false,
        }
    }

    fn target_os_str() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "freebsd") {
            "freebsd"
        } else if cfg!(target_os = "android") {
            "android"
        } else if cfg!(target_os = "ios") {
            "ios"
        } else {
            "unknown"
        }
    }

    fn target_arch_str() -> &'static str {
        if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "wasm32") {
            "wasm32"
        } else {
            "unknown"
        }
    }

    fn target_family_str() -> &'static str {
        if cfg!(target_family = "windows") || cfg!(windows) {
            "windows"
        } else if cfg!(target_family = "unix") || cfg!(unix) {
            "unix"
        } else {
            "unknown"
        }
    }

    fn target_env_str() -> &'static str {
        if cfg!(target_env = "msvc") {
            "msvc"
        } else if cfg!(target_env = "gnu") {
            "gnu"
        } else if cfg!(target_env = "musl") {
            "musl"
        } else {
            ""
        }
    }

    fn target_pointer_width_str() -> &'static str {
        if cfg!(target_pointer_width = "64") {
            "64"
        } else if cfg!(target_pointer_width = "32") {
            "32"
        } else if cfg!(target_pointer_width = "16") {
            "16"
        } else {
            "unknown"
        }
    }

    // grammar:
    //
    //   expr    := and ('|' and)*
    //   and     := pred (',' pred)*          -- comma = AND
    //   pred    := 'cfg(' expr ')'
    //            | 'all(' arglist ')'
    //            | 'any(' arglist ')'
    //            | 'not(' pred ')'
    //            | bare_ident
    //            | ident '=' string_lit      -- string comparison
    //
    //   arglist := pred (',' pred)*

    fn parse_or<'a>(&'a self, input: &'a str) -> Result<(&'a str, bool), ()> {
        let (mut rest, mut acc) = self.parse_and(input)?;
        loop {
            rest = rest.trim_start();
            if rest.starts_with('|') {
                rest = rest[1..].trim_start();
                let (r, val) = self.parse_and(rest)?;
                acc = acc || val;
                rest = r;
            } else {
                break;
            }
        }
        Ok((rest, acc))
    }

    fn parse_and<'a>(&'a self, input: &'a str) -> Result<(&'a str, bool), ()> {
        let (mut rest, mut acc) = self.parse_pred(input)?;
        loop {
            rest = rest.trim_start();
            if rest.starts_with(',') {
                rest = rest[1..].trim_start();
                let (r, val) = self.parse_pred(rest)?;
                acc = acc && val;
                rest = r;
            } else {
                break;
            }
        }
        Ok((rest, acc))
    }

    fn parse_pred<'a>(&'a self, input: &'a str) -> Result<(&'a str, bool), ()> {
        let input = input.trim_start();
        if input.starts_with("cfg(") {
            let (rest, val) = self.parse_group(&input[4..])?;
            Ok((rest, val))
        } else if input.starts_with("all(") {
            let (rest, vals) = self.parse_arg_list(&input[4..])?;
            Ok((rest, vals.iter().all(|v| *v)))
        } else if input.starts_with("any(") {
            let (rest, vals) = self.parse_arg_list(&input[4..])?;
            Ok((rest, vals.iter().any(|v| *v)))
        } else if input.starts_with("not(") {
            let (rest, val) = self.parse_group(&input[4..])?;
            Ok((rest, !val))
        } else {
            let end = input
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(input.len());
            let ident = &input[..end];
            let after_ident = input[end..].trim_start();
            if after_ident.starts_with('=') {
                // String comparison: ident = "value"
                let rest = after_ident[1..].trim_start();
                let val = self.compare_value(ident, rest)?;
                let rest = Self::skip_string_literal(rest)?;
                Ok((rest, val))
            } else {
                Ok((&input[end..], self.lookup_bare(ident)))
            }
        }
    }

    fn parse_group<'a>(&'a self, input: &'a str) -> Result<(&'a str, bool), ()> {
        let (rest, val) = self.parse_or(input)?;
        let rest = rest.trim_start();
        if !rest.starts_with(')') {
            return Err(());
        }
        Ok((&rest[1..], val))
    }

    fn parse_arg_list<'a>(&'a self, input: &'a str) -> Result<(&'a str, Vec<bool>), ()> {
        let mut results = Vec::new();
        let mut rest = input.trim_start();
        if rest.starts_with(')') {
            return Ok((&rest[1..], results));
        }
        loop {
            let (r, val) = self.parse_pred(rest)?;
            results.push(val);
            rest = r.trim_start();
            if rest.starts_with(')') {
                return Ok((&rest[1..], results));
            }
            if !rest.starts_with(',') {
                return Err(());
            }
            rest = rest[1..].trim_start();
        }
    }

    fn compare_value(&self, key: &str, rest: &str) -> Result<bool, ()> {
        let rest = rest.trim_start();
        if !rest.starts_with('"') {
            return Err(());
        }
        let rest = &rest[1..]; // skip opening "
        let end = rest.find('"').ok_or(())?;
        let expected = &rest[..end];
        match self.values.get(key) {
            Some(Value::Str(v)) => Ok(v == expected),
            Some(Value::Bool(_)) => Err(()), // can't compare bool with string
            None => Ok(false),
        }
    }

    fn skip_string_literal(rest: &str) -> Result<&str, ()> {
        let rest = rest.trim_start();
        if !rest.starts_with('"') {
            return Err(());
        }
        let rest = &rest[1..];
        let end = rest.find('"').ok_or(())?;
        Ok(&rest[end + 1..])
    }

    fn lookup_bare(&self, ident: &str) -> bool {
        match self.values.get(ident) {
            Some(Value::Bool(b)) => *b,
            _ => false,
        }
    }

    pub fn lookup(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl Default for CfgEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
