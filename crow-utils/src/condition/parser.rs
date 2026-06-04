use std::collections::VecDeque;
use crate::condition::Condition;

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
            ' ' | '\t' | '\n' | '\r' => { i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '=' => { tokens.push(Token::Eq); i += 1; }
            '"' => {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && chars[end] != '"' {
                    if chars[end] == '\\' && end + 1 < chars.len() { end += 1; }
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
        Self { tokens: tokens.into() }
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
                    Ok(Condition::Cfg { key, value: Some(value) })
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
        Err(format!("trailing tokens after condition: {:?}", parser.tokens))
    }
}