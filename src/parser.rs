use crate::prelude::*;

use serde_json::Map;
use serde_json::Number;
use serde_json::Value;
use std::str::FromStr;

use crate::token::DelimiterKind;
use crate::token::IdentifierKind;
use crate::token::LiteralKind;
use crate::token::TokenKind;

pub fn bytes_to_str(_bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(_bytes) }
}

pub fn bytes_to_string(_bytes: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(_bytes.to_vec()) }
}

pub fn unescape_bytes(_bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(_bytes.len());
    let mut chars = _bytes.iter();

    while let Some(&c) = chars.next() {
        match c {
            b'\\' => {
                if let Some(&b) = chars.next() {
                    match b {
                        b'n' => output.push(b'\n'),
                        b'r' => output.push(b'\r'),
                        b't' => output.push(b'\t'),
                        b'"' => output.push(b),
                        b'\'' => output.push(b),
                        b'\\' => output.push(b),
                        b'`' => output.push(b),
                        _ => output.push(b),
                    }
                }
            }
            _ => output.push(c),
        }
    }

    output
}

#[derive(Default, Debug)]
pub struct Parser {
    index: usize,
}

impl Parser {
    fn index(&self) -> usize {
        self.index
    }

    fn next(&mut self) {
        self.index += 1;
    }

    pub fn new() -> Self {
        Parser { index: 0 }
    }

    pub fn parse(&mut self, tokens: &[TokenKind]) -> Result<Value> {
        match tokens.get(self.index()) {
            Some(token) => match token {
                TokenKind::Delimiter(DelimiterKind::TablePrec) => self.create_table(tokens),

                TokenKind::Delimiter(DelimiterKind::ListPrec) => self.create_list(tokens),

                TokenKind::Identifier(IdentifierKind::String(_)) => {
                    let mut values = Map::with_capacity(tokens.len());

                    while tokens.get(self.index()).is_some() {
                        let key = self.create_key(tokens)?;
                        self.next();
                        let value = self.create_value(tokens)?;
                        self.next();

                        values.insert(key, value);
                    }

                    Ok(Value::Object(values))
                }
                _ => {
                    let mut values = Vec::with_capacity(tokens.len());

                    while tokens.get(self.index()).is_some() {
                        let value = self.create_value(tokens)?;
                        self.next();

                        values.push(value);
                    }

                    Ok(Value::Array(values))
                }
            },
            None => Err(Error {
                desc: "ran out of tokens".to_string(),
            }),
        }
    }

    pub fn create_list<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> Result<Value> {
        let mut values = Vec::new();

        while let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Delimiter(DelimiterKind::ListTerm) => break,
                _ => {
                    let value = self.create_value(tokens)?;

                    self.next();

                    values.push(value);
                }
            }
        }

        Ok(Value::Array(values))
    }

    pub fn create_table<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> Result<Value> {
        let mut values = Map::new();

        while let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Delimiter(DelimiterKind::TableTerm) => break,

                _ => {
                    let key = self.create_key(tokens)?;
                    self.next();

                    let value = self.create_value(tokens)?;
                    self.next();

                    values.insert(key, value);
                }
            }
        }

        Ok(Value::Object(values))
    }

    pub fn create_key<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> Result<String> {
        if let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Literal(LiteralKind::String(t)) => {
                    let result = bytes_to_string(t.bytes());
                    Ok(result)
                }

                TokenKind::Identifier(IdentifierKind::String(t)) => {
                    let result = bytes_to_string(t.bytes());
                    Ok(result)
                }

                token => Err(Error {
                    desc: format!("invalid key '{:?}'", token),
                }),
            }
        } else {
            Err(Error {
                desc: "expected a key".to_string(),
            })
        }
    }

    pub fn create_value<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> Result<Value> {
        if let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Literal(LiteralKind::True) => Ok(Value::Bool(true)),

                TokenKind::Literal(LiteralKind::False) => Ok(Value::Bool(false)),

                TokenKind::Literal(LiteralKind::String(t)) => {
                    let bytes_str = t.bytes();

                    if bytes_str.contains(&b'\\') {
                        let unescaped = unescape_bytes(bytes_str);
                        let result = bytes_to_string(&unescaped);
                        Ok(Value::String(result))
                    } else {
                        let result = bytes_to_string(bytes_str);
                        Ok(Value::String(result))
                    }
                }

                TokenKind::Literal(LiteralKind::Number(t)) => {
                    let num_str = bytes_to_str(t.bytes());

                    match Number::from_str(num_str) {
                        Ok(num) => Ok(Value::Number(num)),
                        Err(e) => Err(Error {
                            desc: e.to_string(),
                        }),
                    }
                }

                TokenKind::Literal(LiteralKind::Null) => Ok(Value::Null),

                TokenKind::Delimiter(DelimiterKind::TablePrec) => {
                    self.next(); // skip opening "{"

                    match self.create_table(tokens) {
                        Ok(tbl) => Ok(tbl),
                        Err(e) => Err(Error {
                            desc: format!("failed creating a table because of {}", e.desc),
                        }),
                    }
                }

                TokenKind::Delimiter(DelimiterKind::ListPrec) => {
                    self.next(); // skip opening "["

                    match self.create_list(tokens) {
                        Ok(ls) => Ok(ls),
                        Err(e) => Err(Error {
                            desc: format!("failed creating a list because of {}", e.desc),
                        }),
                    }
                }

                token => Err(Error {
                    desc: format!("invalid value '{:?}'", token),
                }),
            }
        } else {
            Err(Error {
                desc: "ran out of tokens".to_string(),
            })
        }
    }
}
