use std::collections::VecDeque;
use target_lexicon::{Architecture, OperatingSystem, Triple};
use toml_edit::{DocumentMut, Item, Table};

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub triple: Triple,
    pub os: String,
    pub arch: String,
    pub vendor: String,
    pub env: String,
    pub pointer_width: u8,
    pub family: Option<String>,
}

impl TargetInfo {
    pub fn current() -> Self {
        let triple = Triple::host();
        let os = triple.operating_system.to_string();
        let arch = triple.architecture.to_string();
        let vendor = triple.vendor.to_string();
        let env = triple.environment.to_string();
        let pointer_width = match triple.architecture {
            Architecture::X86_64 | Architecture::Aarch64(_) => 64,
            Architecture::X86_32(_) | Architecture::Arm(_) => 32,
            _ => 64,
        };
        let family = match triple.operating_system {
            OperatingSystem::Windows => Some("windows".to_string()),
            OperatingSystem::Linux
            | OperatingSystem::MacOSX { .. }
            | OperatingSystem::Freebsd
            | OperatingSystem::Netbsd
            | OperatingSystem::Openbsd
            | OperatingSystem::Dragonfly => Some("unix".to_string()),
            _ => None,
        };
        Self {
            triple,
            os,
            arch,
            vendor,
            env,
            pointer_width,
            family,
        }
    }

    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }
    pub fn is_unix(&self) -> bool {
        self.family.as_deref() == Some("unix")
    }
    pub fn is_macos(&self) -> bool {
        self.os == "macos"
    }
    pub fn is_linux(&self) -> bool {
        self.os == "linux"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Cfg { key: String, value: Option<String> },
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    Triple(String),
}

#[derive(Debug, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
    LParen,
    RParen,
    Comma,
    Eq,
    Eof,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Eq);
                i += 1;
            }
            '"' => {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && chars[end] != '"' {
                    if chars[end] == '\\' && end + 1 < chars.len() {
                        end += 1;
                    }
                    end += 1;
                }
                tokens.push(Token::Str(chars[start..end].iter().collect()));
                i = end + 1;
            }
            _ => {
                let start = i;
                while i < chars.len()
                    && matches!(chars[i], 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.')
                {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
                // no i += 1: already advanced by inner loop
            }
        }
    }

    tokens.push(Token::Eof);
    tokens
}

struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into(),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.front().unwrap_or(&Token::Eof)
    }

    fn consume(&mut self) -> Token {
        self.tokens.pop_front().unwrap_or(Token::Eof)
    }

    fn expect_ident(&mut self, expected: &str) -> Result<(), String> {
        match self.consume() {
            Token::Ident(s) if s == expected => Ok(()),
            tok => Err(format!("expected '{}', got {:?}", expected, tok)),
        }
    }

    fn expect_lparen(&mut self) -> Result<(), String> {
        match self.consume() {
            Token::LParen => Ok(()),
            tok => Err(format!("expected '(', got {:?}", tok)),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.consume() {
            Token::RParen => Ok(()),
            tok => Err(format!("expected ')', got {:?}", tok)),
        }
    }

    fn parse_condition(&mut self) -> Result<Condition, String> {
        match self.peek() {
            Token::Ident(s) if s == "cfg" => self.parse_cfg_wrapper(),
            Token::Ident(s) if s == "all" => self.parse_all(),
            Token::Ident(s) if s == "any" => self.parse_any(),
            Token::Ident(s) if s == "not" => self.parse_not(),
            _ => self.parse_triple(),
        }
    }

    fn parse_cfg_wrapper(&mut self) -> Result<Condition, String> {
        self.expect_ident("cfg")?;
        self.expect_lparen()?;
        let inner = self.parse_cfg_predicate()?;
        self.expect_rparen()?;
        Ok(inner)
    }

    fn parse_cfg_predicate(&mut self) -> Result<Condition, String> {
        match self.peek() {
            Token::Ident(s) if !matches!(s.as_str(), "all" | "any" | "not") => {
                let key = match self.consume() {
                    Token::Ident(k) => k,
                    _ => unreachable!(),
                };
                if matches!(self.peek(), Token::Eq) {
                    self.consume();
                    let value = match self.consume() {
                        Token::Str(v) | Token::Ident(v) => v,
                        tok => return Err(format!("expected value after '=', got {:?}", tok)),
                    };
                    Ok(Condition::Cfg {
                        key,
                        value: Some(value),
                    })
                } else {
                    Ok(Condition::Cfg { key, value: None })
                }
            }
            Token::Ident(s) if matches!(s.as_str(), "all" | "any" | "not") => {
                self.parse_composite()
            }
            tok => Err(format!("expected cfg predicate, got {:?}", tok)),
        }
    }

    fn parse_composite(&mut self) -> Result<Condition, String> {
        match self.peek() {
            Token::Ident(s) if s == "all" => self.parse_all(),
            Token::Ident(s) if s == "any" => self.parse_any(),
            Token::Ident(s) if s == "not" => self.parse_not(),
            tok => Err(format!("expected 'all', 'any', or 'not', got {:?}", tok)),
        }
    }

    fn parse_predicate_list(&mut self) -> Result<Vec<Condition>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            self.consume();
            return Ok(args);
        }
        loop {
            args.push(self.parse_cfg_predicate()?);
            match self.consume() {
                Token::Comma => continue,
                Token::RParen => break,
                tok => return Err(format!("expected ',' or ')', got {:?}", tok)),
            }
        }
        Ok(args)
    }

    fn parse_all(&mut self) -> Result<Condition, String> {
        self.expect_ident("all")?;
        self.expect_lparen()?;
        Ok(Condition::All(self.parse_predicate_list()?))
    }

    fn parse_any(&mut self) -> Result<Condition, String> {
        self.expect_ident("any")?;
        self.expect_lparen()?;
        Ok(Condition::Any(self.parse_predicate_list()?))
    }

    fn parse_not(&mut self) -> Result<Condition, String> {
        self.expect_ident("not")?;
        self.expect_lparen()?;
        let inner = self.parse_cfg_predicate()?;
        self.expect_rparen()?;
        Ok(Condition::Not(Box::new(inner)))
    }

    fn parse_triple(&mut self) -> Result<Condition, String> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Token::Ident(_) | Token::Str(_) => {
                    parts.push(match self.consume() {
                        Token::Ident(s) | Token::Str(s) => s,
                        _ => unreachable!(),
                    });
                }
                _ => break,
            }
        }
        if parts.is_empty() {
            Err("expected triple or cfg expression".to_string())
        } else {
            Ok(Condition::Triple(parts.join("")))
        }
    }
}

pub fn parse_condition(input: &str) -> Result<Condition, String> {
    let mut parser = Parser::new(tokenize(input));
    let cond = parser.parse_condition()?;
    if parser.tokens.iter().all(|t| matches!(t, Token::Eof)) {
        Ok(cond)
    } else {
        Err(format!(
            "trailing tokens after condition: {:?}",
            parser.tokens
        ))
    }
}

fn eval_cfg(key: &str, value: Option<&str>, target: &TargetInfo) -> bool {
    match (key, value) {
        ("windows", None) => target.is_windows(),
        ("unix", None) => target.is_unix(),
        ("macos", None) => target.is_macos(),
        ("linux", None) => target.is_linux(),
        ("target_os", Some(v)) => v == target.os,
        ("target_arch", Some(v)) => v == target.arch,
        ("target_vendor", Some(v)) => v == target.vendor,
        ("target_env", Some(v)) => v == target.env,
        ("target_pointer_width", Some(v)) => v == target.pointer_width.to_string(),
        ("target_endian", Some(v)) => {
            if cfg!(target_endian = "little") {
                v == "little"
            } else {
                v == "big"
            }
        }
        ("target_family", Some(v)) => target.family.as_deref() == Some(v),
        _ => false,
    }
}

pub fn eval_condition(cond: &Condition, target: &TargetInfo) -> bool {
    match cond {
        Condition::Cfg { key, value } => eval_cfg(key, value.as_deref(), target),
        Condition::All(conds) => conds.iter().all(|c| eval_condition(c, target)),
        Condition::Any(conds) => conds.iter().any(|c| eval_condition(c, target)),
        Condition::Not(c) => !eval_condition(c, target),
        Condition::Triple(triple_str) => target.triple.to_string() == *triple_str,
    }
}

fn merge_item(base: &mut Item, overlay: &Item) {
    match (base, overlay) {
        (Item::Table(base_tab), Item::Table(overlay_tab)) => {
            for (k, v) in overlay_tab.iter() {
                match base_tab.get_mut(k) {
                    Some(existing) => merge_item(existing, v),
                    None => {
                        base_tab.insert(k, v.clone());
                    }
                }
            }
        }
        (base_item, _) => *base_item = overlay.clone(),
    }
}

fn merge_table_into_root(root: &mut DocumentMut, section_name: &str, overlay: &Table) {
    let root_item = root
        .entry(section_name)
        .or_insert(Item::Table(Table::new()));
    if let Some(root_tab) = root_item.as_table_mut() {
        for (k, v) in overlay.iter() {
            match root_tab.get_mut(k) {
                Some(existing) => merge_item(existing, v),
                None => {
                    root_tab.insert(k, v.clone());
                }
            }
        }
    }
}

fn collect_matching_sections(
    doc: &DocumentMut,
    target: &TargetInfo,
) -> Result<Vec<(String, Table)>, String> {
    let target_table = match doc.get("target").and_then(|item| item.as_table()) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for (raw_key, value) in target_table.iter() {
        let cond_str = raw_key.trim_matches('"');

        let cond = parse_condition(cond_str)
            .map_err(|e| format!("failed to parse condition '{}': {}", cond_str, e))?;

        if !eval_condition(&cond, target) {
            continue;
        }

        if let Some(section_table) = value.as_table() {
            result.push((cond_str.to_string(), section_table.clone()));
        }
    }

    Ok(result)
}

fn apply_matching_sections(doc: &mut DocumentMut, sections: Vec<(String, Table)>) {
    for (_cond_str, cond_table) in sections {
        for (section_name, section_value) in cond_table.iter() {
            match section_value.as_table() {
                Some(overlay_tab) => merge_table_into_root(doc, section_name, overlay_tab),
                None => {
                    *doc.entry(section_name).or_insert(section_value.clone()) =
                        section_value.clone();
                }
            }
        }
    }
}

pub fn apply_target_filter(content: &str, target: &TargetInfo) -> Result<String, String> {
    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| format!("failed to parse TOML: {}", e))?;

    let matching = collect_matching_sections(&doc, target)?;
    apply_matching_sections(&mut doc, matching);
    doc.remove("target");

    Ok(doc.to_string())
}
