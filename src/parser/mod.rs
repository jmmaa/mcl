use crate::tokenizer::Token;

use ahash::AHashMap;

use std::{error::Error, fmt::Display};

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum KeyKind<'a> {
    String(&'a String),
    Integer(&'a i64),
    Str(&'a str),
}

#[derive(Debug)]
pub enum ValueKind<'a> {
    Table(AHashMap<KeyKind<'a>, ValueKind<'a>>),
    List(Vec<ValueKind<'a>>),
    String(&'a String),
    Boolean(&'a bool),
    Integer(&'a i64),
    Float(&'a f64),
    Str(&'a str),
}

#[derive(Debug)]
pub struct ParserError {
    pub description: String,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Parser { tokens, pos: 0 }
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn create_tree(&mut self) -> Result<ValueKind<'a>, ParserError> {
        let token = self.tokens.get(self.pos); // peek

        match token {
            Some(Token::TablePrec) => match self.create_table() {
                Ok(tbl) => Ok(ValueKind::Table(tbl)),
                Err(e) => Err(e),
            },
            Some(Token::ListPrec) => match self.create_list() {
                Ok(ls) => Ok(ValueKind::List(ls)),
                Err(e) => Err(e),
            },
            Some(Token::Keyword(_)) => {
                let mut values = AHashMap::new();

                while self.tokens.get(self.pos).is_some() {
                    match self.create_key_value_pair() {
                        Ok((k, v)) => {
                            values.insert(k, v);
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok(ValueKind::Table(values))
            }
            Some(_) => {
                let mut values = Vec::new();

                while self.tokens.get(self.pos).is_some() {
                    match self.create_value() {
                        Ok(v) => {
                            values.push(v);
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok(ValueKind::List(values))
            }
            None => Err(ParserError {
                description: "ran out of tokens".to_string(),
            }),
        }
    }

    #[inline(always)]
    pub fn create_list(&mut self) -> Result<Vec<ValueKind<'a>>, ParserError> {
        let mut values = Vec::new();

        while let Some(b) = self.tokens.get(self.pos) {
            match b {
                Token::ListTerm => {
                    self.advance();
                    break;
                }
                _ => match self.create_value() {
                    Ok(v) => {
                        values.push(v);
                    }
                    Err(e) => return Err(e),
                },
            }
        }

        Ok(values)
    }

    #[inline(always)]
    pub fn create_table(&mut self) -> Result<AHashMap<KeyKind<'a>, ValueKind<'a>>, ParserError> {
        let mut values = AHashMap::new();

        while let Some(b) = self.tokens.get(self.pos) {
            match b {
                Token::TableTerm => {
                    self.advance();
                    break;
                }
                _ => match self.create_key_value_pair() {
                    Ok((k, v)) => {
                        values.insert(k, v);
                    }
                    Err(e) => return Err(e),
                },
            }
        }

        Ok(values)
    }

    #[inline(always)]
    pub fn create_key_value_pair(&mut self) -> Result<(KeyKind<'a>, ValueKind<'a>), ParserError> {
        if let Some(b) = self.tokens.get(self.pos) {
            match b {
                Token::Keyword(key) => {
                    self.advance();

                    match self.create_value() {
                        Ok(value) => Ok((KeyKind::Str(key), value)),
                        Err(e) => Err(e),
                    }
                }
                Token::String(key) => {
                    self.advance();

                    match self.create_value() {
                        Ok(value) => Ok((KeyKind::String(key), value)),
                        Err(e) => Err(e),
                    }
                }

                Token::Integer(key) => {
                    self.advance();

                    match self.create_value() {
                        Ok(value) => Ok((KeyKind::Integer(key), value)),
                        Err(e) => Err(e),
                    }
                }

                token => Err(ParserError {
                    description: format!("invalid key '{:?}' for key-value-pair", token),
                }),
            }
        } else {
            Err(ParserError {
                description: "expected a key".to_string(),
            })
        }
    }

    #[inline(always)]
    pub fn create_value(&mut self) -> Result<ValueKind<'a>, ParserError> {
        if let Some(token) = self.tokens.get(self.pos) {
            self.advance();

            match token {
                Token::Float(value) => Ok(ValueKind::Float(value)),

                Token::Boolean(value) => Ok(ValueKind::Boolean(value)),

                Token::Integer(value) => Ok(ValueKind::Integer(value)),

                Token::String(value) => Ok(ValueKind::String(value)),

                Token::Str(value) => Ok(ValueKind::Str(value)),

                Token::TablePrec => match self.create_table() {
                    Ok(table) => Ok(ValueKind::Table(table)),
                    Err(e) => Err(ParserError {
                        description: format!(
                            "failed creating a table because of {}",
                            e.description
                        ),
                    }),
                },
                Token::ListPrec => match self.create_list() {
                    Ok(ls) => Ok(ValueKind::List(ls)),
                    Err(e) => Err(ParserError {
                        description: format!("failed creating a list because of {}", e.description),
                    }),
                },

                _ => Err(ParserError {
                    description: format!("invalid value '{:?}'", token),
                }),
            }
        } else {
            Err(ParserError {
                description: "ran out of tokens".to_string(),
            })
        }
    }
}
