use crate::tokenizer::{DelimiterKind, KeywordKind, LiteralKind, TokenKind};
use std::collections::HashMap;

use std::{error::Error, fmt::Display};

#[inline(always)]
fn bytes_slice_to_str(_bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(_bytes) }
}

#[inline(always)]
fn bytes_slice_to_string(_bytes: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(_bytes.to_vec()) }
}

#[inline(always)]
fn bytes_to_float(_bytes: &[u8]) -> ParserResult<f64> {
    let float_str = bytes_slice_to_str(_bytes);

    match float_str.parse::<f64>() {
        Ok(v) => Ok(v),
        Err(e) => Err(ParserError {
            desc: format!("error converting to float: {:?} value: {float_str}", e),
        }),
    }
}

#[inline(always)]
fn bytes_to_int(_bytes: &[u8]) -> ParserResult<i64> {
    let int_str = bytes_slice_to_str(_bytes);

    match int_str.parse::<i64>() {
        Ok(v) => Ok(v),
        Err(e) => Err(ParserError {
            desc: format!("error converting to float: {:?} value: {int_str}", e),
        }),
    }
}

#[inline(always)]
fn unescape_bytes_string(_bytes: &[u8]) -> Vec<u8> {
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

#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(List),
    Table(Table),
}

pub type List = Vec<Value>;

pub type Table = HashMap<String, Value>;

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub struct ParserError {
    pub desc: String,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        self.desc.as_str()
    }
}

#[derive(Default, Debug)]
pub struct Parser {
    index: usize,
}

impl Parser {
    #[inline]
    fn index(&self) -> usize {
        self.index
    }

    #[inline]
    fn next(&mut self) {
        self.index += 1;
    }

    pub fn new() -> Self {
        Parser { index: 0 }
    }

    pub fn parse(&mut self, tokens: &[TokenKind]) -> ParserResult<Value> {
        match tokens.get(self.index()) {
            Some(token) => match token {
                TokenKind::Delimiter(DelimiterKind::TablePrec) => match self.create_table(tokens) {
                    Ok(tbl) => Ok(Value::Table(tbl)),
                    Err(e) => Err(e),
                },
                TokenKind::Delimiter(DelimiterKind::ListPrec) => match self.create_list(tokens) {
                    Ok(ls) => Ok(Value::List(ls)),
                    Err(e) => Err(e),
                },
                TokenKind::Keyword(KeywordKind::String(_)) => {
                    let mut values = HashMap::new();

                    while tokens.get(self.index()).is_some() {
                        let key = self.create_key(tokens)?;
                        self.next();
                        let value = self.create_value(tokens)?;
                        self.next();

                        values.insert(key, value);
                    }

                    Ok(Value::Table(values))
                }
                _ => {
                    let mut values = Vec::new();

                    while tokens.get(self.index()).is_some() {
                        let value = self.create_value(tokens)?;

                        self.next();

                        values.push(value);
                    }

                    Ok(Value::List(values))
                }
            },
            None => Err(ParserError {
                desc: "ran out of tokens".to_string(),
            }),
        }
    }

    pub fn create_value<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> Result<Value, ParserError> {
        if let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Keyword(KeywordKind::True(_)) => Ok(Value::Boolean(true)),

                TokenKind::Keyword(KeywordKind::String(tok)) => {
                    if tok.raw.contains(&b'\\') {
                        let unescaped = unescape_bytes_string(tok.raw);
                        let result = bytes_slice_to_string(&unescaped);
                        Ok(Value::String(result))
                    } else {
                        let result = bytes_slice_to_string(tok.raw);
                        Ok(Value::String(result))
                    }
                }

                TokenKind::Literal(LiteralKind::String(tok)) => {
                    if tok.raw.contains(&b'\\') {
                        let unescaped = unescape_bytes_string(tok.raw);
                        let result = bytes_slice_to_string(&unescaped);
                        Ok(Value::String(result))
                    } else {
                        let result = bytes_slice_to_string(tok.raw);
                        Ok(Value::String(result))
                    }
                }

                TokenKind::Literal(LiteralKind::Float(tok)) => match bytes_to_float(tok.raw) {
                    Ok(v) => Ok(Value::Float(v)),
                    Err(e) => Err(e),
                },

                TokenKind::Literal(LiteralKind::Int(tok)) => match bytes_to_int(tok.raw) {
                    Ok(v) => Ok(Value::Integer(v)),
                    Err(e) => Err(e),
                },

                TokenKind::Delimiter(DelimiterKind::TablePrec) => {
                    self.next(); // skip opening "{"

                    match self.create_table(tokens) {
                        Ok(table) => Ok(Value::Table(table)),
                        Err(e) => Err(ParserError {
                            desc: format!("failed creating a table because of {}", e.desc),
                        }),
                    }
                }

                TokenKind::Delimiter(DelimiterKind::ListPrec) => {
                    self.next(); // skip opening "["

                    match self.create_list(tokens) {
                        Ok(ls) => Ok(Value::List(ls)),
                        Err(e) => Err(ParserError {
                            desc: format!("failed creating a list because of {}", e.desc),
                        }),
                    }
                }

                token => Err(ParserError {
                    desc: format!("invalid value '{:?}'", token),
                }),
            }
        } else {
            Err(ParserError {
                desc: "ran out of tokens".to_string(),
            })
        }
    }

    pub fn create_list<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> ParserResult<Vec<Value>> {
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

        Ok(values)
    }

    pub fn create_table<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> ParserResult<Table> {
        let mut values = HashMap::new();

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

        Ok(values)
    }

    pub fn create_key<'a>(&mut self, tokens: &'a [TokenKind<'a>]) -> ParserResult<String> {
        if let Some(token) = tokens.get(self.index()) {
            match token {
                TokenKind::Literal(LiteralKind::String(tok)) => {
                    let result = bytes_slice_to_string(tok.raw);
                    Ok(result)
                }

                TokenKind::Keyword(KeywordKind::String(tok)) => {
                    let result = bytes_slice_to_string(tok.raw);
                    Ok(result)
                }

                token => Err(ParserError {
                    desc: format!("invalid key '{:?}'", token),
                }),
            }
        } else {
            Err(ParserError {
                desc: "expected a key".to_string(),
            })
        }
    }
}
