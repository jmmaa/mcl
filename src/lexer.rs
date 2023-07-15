use crate::prelude::*;

use crate::token::DelimiterKind;
use crate::token::IdentifierKind;
use crate::token::LiteralKind;
use crate::token::Location;
use crate::token::Position;
use crate::token::Token;
use crate::token::TokenKind;

#[derive(Default)]
pub struct Lexer {
    index: usize,
    column: usize,
    line: usize,
}

impl Lexer {
    fn index(&self) -> usize {
        self.index
    }

    fn column(&self) -> usize {
        self.column
    }

    fn line(&self) -> usize {
        self.line
    }

    fn next(&mut self) {
        self.index += 1;
        self.column += 1;
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
        self.index += 1;
    }

    fn position(&self) -> Position {
        Position::new(self.line(), self.column(), self.index())
    }

    fn string<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>> {
        self.next(); // skip opening double quotes

        let start = self.position();

        while let Some(&b) = source.get(self.index()) {
            match b {
                b'"' => break,
                b'\n' => {
                    return Err(Error {
                        desc: "cannot use newline character in strings".to_string(),
                    })
                }
                _ => self.next(),
            }
        }

        if source.get(self.index()).is_none() {
            return Err(Error {
                desc: format!("unterminated string ({}:{})", start.line(), start.column()),
            });
        }

        let end = self.position();

        self.next(); // skip closing double quotes

        let raw = &source[start.index()..end.index()];

        Ok(TokenKind::Literal(LiteralKind::String(Token::new(
            Location::new(start, end),
            raw,
        ))))
    }

    fn identifier<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>> {
        let start = self.position();

        while let Some(b) = source.get(self.index()) {
            if !(b.is_ascii_alphanumeric() || *b == b'_') {
                break;
            }

            self.next();
        }

        let end = self.position();

        let raw = &source[start.index()..end.index()];

        match raw {
            b"true" => Ok(TokenKind::Literal(LiteralKind::True)),

            b"false" => Ok(TokenKind::Literal(LiteralKind::False)),

            b"null" => Ok(TokenKind::Literal(LiteralKind::Null)),

            _ => Ok(TokenKind::Identifier(IdentifierKind::String(Token::new(
                Location::new(start, end),
                raw,
            )))),
        }
    }

    fn number<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>> {
        let mut point = false;
        let mut zero = false;

        if let Some(&b) = source.get(self.index()) {
            if b == b'0' {
                zero = true;
            }
        }

        let start = self.position();

        self.next(); // skip opening first digit

        while let Some(&b) = source.get(self.index()) {
            if b.is_ascii_digit() && !zero {
                self.next()
            } else if b == b'.' && !point {
                point = true;
                zero = false;

                self.next();

                if let Some(b) = source.get(self.index()) {
                    if !b.is_ascii_digit() {
                        return Err(Error {
                            desc: format!(
                                "decimal point must be followed with a digit, not '{}' ({}:{})",
                                *b as char,
                                self.line(),
                                self.column(),
                            ),
                        });
                    }
                } else {
                    return Err(Error {
                        desc: "decimal point must be followed with a digit, but no bytes left"
                            .to_string(),
                    });
                }
            } else {
                break;
            }
        }

        let end = self.position();
        let raw = &source[start.index()..end.index()];

        Ok(TokenKind::Literal(LiteralKind::Number(Token::new(
            Location::new(start, end),
            raw,
        ))))
    }

    fn template_string<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>> {
        self.next(); // skip opening tilde

        let start = self.position();

        while let Some(&b) = source.get(self.index()) {
            match b {
                b'\\' => {
                    // escape char
                    self.next();
                    self.next();
                }
                b'\n' => {
                    self.next_line();
                }
                b'`' => break,
                _ => {
                    self.next();
                }
            }
        }

        if source.get(self.index()).is_none() {
            return Err(Error {
                desc: format!(
                    "unterminated template string ({}:{})",
                    start.line(),
                    start.column()
                ),
            });
        }

        let end = self.position();

        self.next(); // skip closing tilde

        let raw = &source[start.index()..end.index()];

        Ok(TokenKind::Literal(LiteralKind::String(Token::new(
            Location::new(start, end),
            raw,
        ))))
    }

    fn ignore_comment(&mut self, source: &[u8]) -> Result<()> {
        let start = self.position();

        self.next(); // skip identifier forward slash

        loop {
            if let Some(&b) = source.get(self.index()) {
                if b == b'\n' {
                    break;
                } else {
                    self.next();
                }
            } else {
                return Err(Error {
                    desc: format!("unterminated comment ({}:{})", start.line(), start.column()),
                });
            }
        }

        self.next_line();

        Ok(())
    }

    fn ignore_multiline_comment(&mut self, source: &[u8]) -> Result<()> {
        let start = self.position();

        self.next(); // skip preceding opening slash

        loop {
            if let Some(&b) = source.get(self.index()) {
                if b == b'*' {
                    self.next();
                    if let Some(b'/') = source.get(self.index()) {
                        // skip closing right slash
                        self.next();
                        break;
                    }
                } else if b == b'\n' {
                    self.next_line();
                } else {
                    self.next();
                }
            } else {
                return Err(Error {
                    desc: format!("unterminated comment ({}:{})", start.line(), start.column()),
                });
            }
        }

        Ok(())
    }

    fn comment(&mut self, source: &[u8]) -> Result<()> {
        let start = self.position(); // save start position

        self.next(); // skip preceding opening slash

        match source.get(self.index()) {
            Some(b'/') => match self.ignore_comment(source) {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },

            Some(b'*') => match self.ignore_multiline_comment(source) {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            Some(c) => Err(Error {
                desc: format!(
                    "expected '/' or '*' not '{}' ({}:{})",
                    *c as char,
                    self.line(),
                    self.column()
                ),
            }),

            None => Err(Error {
                desc: format!(
                    "expected '/' or '*' but no bytes left ({}:{})",
                    start.line(),
                    start.column()
                ),
            }),
        }
    }

    pub fn new() -> Self {
        Lexer {
            index: 0,
            column: 1,
            line: 1,
        }
    }

    pub fn tokenize<'a>(&mut self, source: &'a [u8]) -> Result<Vec<TokenKind<'a>>> {
        let mut tokens = Vec::with_capacity(source.len());

        while let Some(b) = source.get(self.index()) {
            match b {
                b'{' => {
                    tokens.push(TokenKind::Delimiter(DelimiterKind::TablePrec));
                    self.next();
                }
                b'}' => {
                    tokens.push(TokenKind::Delimiter(DelimiterKind::TableTerm));
                    self.next();
                }
                b'[' => {
                    tokens.push(TokenKind::Delimiter(DelimiterKind::ListPrec));
                    self.next();
                }
                b']' => {
                    tokens.push(TokenKind::Delimiter(DelimiterKind::ListTerm));
                    self.next();
                }
                b'\n' => self.next_line(),

                // skip whitespaces
                b'\r' | b'\t' | b' ' => self.next(),

                // identifier
                b'a'..=b'z' | b'A'..=b'Z' => match self.identifier(source) {
                    Ok(t) => tokens.push(t),
                    Err(e) => return Err(e),
                },

                // String
                b'"' => match self.string(source) {
                    Ok(t) => tokens.push(t),
                    Err(e) => return Err(e),
                },

                // Template String
                b'`' => match self.template_string(source) {
                    Ok(t) => tokens.push(t),
                    Err(e) => return Err(e),
                },

                // Number
                b'0'..=b'9' | b'+' | b'-' => match self.number(source) {
                    Ok(t) => tokens.push(t),
                    Err(e) => return Err(e),
                },

                // Comments
                b'/' => match self.comment(source) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                _ => {
                    return Err(Error {
                        desc: format!(
                            "unrecognized character '{}' ({}:{})",
                            *b as char,
                            self.line(),
                            self.column(),
                        ),
                    })
                }
            }
        }

        Ok(tokens)
    }
}
