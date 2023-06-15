use std;

use std::error::Error;

#[derive(Debug)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

#[derive(Debug)]
pub struct Loc {
    pub start: Pos,
    pub end: Pos,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub loc: Loc,
    pub raw: &'a [u8],
}

#[derive(Debug)]
pub enum LiteralKind<'a> {
    String(Token<'a>),
    Int(Token<'a>),
    Float(Token<'a>),
    Bool(Token<'a>),
}

#[derive(Debug)]
pub enum KeywordKind<'a> {
    String(Token<'a>),
    True(Token<'a>),
    False(Token<'a>),
}

#[derive(Debug)]
pub enum DelimiterKind {
    TableTerm,
    TablePrec,
    ListPrec,
    ListTerm,
}

#[derive(Debug)]
pub enum TokenKind<'a> {
    Keyword(KeywordKind<'a>),
    Literal(LiteralKind<'a>),
    Delimiter(DelimiterKind),
}

#[derive(Debug)]
pub struct TokenizerError {
    pub desc: String,
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TokenizerError {
    fn description(&self) -> &str {
        self.desc.as_str()
    }
}

#[derive(Default)]
pub struct Tokenizer {
    index: usize,
    column: usize,
    line: usize,
}

impl Tokenizer {
    #[inline]
    fn index(&self) -> usize {
        self.index
    }

    #[inline]
    fn column(&self) -> usize {
        self.column
    }

    #[inline]
    fn line(&self) -> usize {
        self.line
    }

    #[inline]
    fn next(&mut self) {
        self.index += 1;
        self.column += 1;
    }

    #[inline]
    fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
        self.index += 1;
    }

    #[inline]
    fn position(&self) -> Pos {
        Pos {
            line: self.line(),
            column: self.column(),
            index: self.index(),
        }
    }

    fn keyword<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>, TokenizerError> {
        let start = self.position();

        while let Some(b) = source.get(self.index()) {
            if !(b.is_ascii_alphanumeric() || *b == b'_') {
                break;
            }

            self.next();
        }

        let end = self.position();

        let raw = &source[start.index..end.index];

        match raw {
            b"true" => Ok(TokenKind::Keyword(KeywordKind::True(Token {
                loc: Loc { start, end },
                raw,
            }))),

            b"false" => Ok(TokenKind::Keyword(KeywordKind::False(Token {
                loc: Loc { start, end },
                raw,
            }))),

            _ => Ok(TokenKind::Keyword(KeywordKind::String(Token {
                loc: Loc { start, end },
                raw,
            }))),
        }
    }

    fn number<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>, TokenizerError> {
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
                        return Err(TokenizerError {
                            desc: format!("decimal point must be followed with a digit, not {}", b),
                        });
                    }
                } else {
                    return Err(TokenizerError {
                        desc: "decimal point must be followed with a digit, but no bytes left"
                            .to_string(),
                    });
                }
            } else {
                break;
            }
        }

        let end = self.position();
        let raw = &source[start.index..end.index];

        if point {
            Ok(TokenKind::Literal(LiteralKind::Float(Token {
                loc: Loc { start, end },
                raw,
            })))
        } else {
            Ok(TokenKind::Literal(LiteralKind::Int(Token {
                loc: Loc { start, end },
                raw,
            })))
        }
    }

    fn template_string<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>, TokenizerError> {
        self.next(); // skip opening tilde

        let start = self.position();

        while let Some(&b) = source.get(self.index()) {
            match b {
                b'\\' => {
                    // escape char
                    self.next();
                    self.next();
                }
                b'`' => break,
                _ => {
                    self.next();
                }
            }
        }

        if source.get(self.index()).is_none() {
            return Err(TokenizerError {
                desc: "unterminated mutli line string".to_string(),
            });
        }

        let end = self.position();

        self.next(); // skip closing tilde

        let raw = &source[start.index..end.index];

        Ok(TokenKind::Literal(LiteralKind::String(Token {
            loc: Loc { start, end },
            raw,
        })))
    }

    fn string<'a>(&mut self, source: &'a [u8]) -> Result<TokenKind<'a>, TokenizerError> {
        self.next(); // skip opening double quotes

        let start = self.position();

        while let Some(&b) = source.get(self.index()) {
            match b {
                b'"' => break,
                b'\n' => {
                    return Err(TokenizerError {
                        desc: "cannot use newline character in strings".to_string(),
                    })
                }
                _ => self.next(),
            }
        }

        if source.get(self.index()).is_none() {
            return Err(TokenizerError {
                desc: "unterminated string".to_string(),
            });
        }

        let end = self.position();

        self.next(); // skip closing double quotes

        let raw = &source[start.index..end.index];

        Ok(TokenKind::Literal(LiteralKind::String(Token {
            loc: Loc { start, end },
            raw,
        })))
    }

    fn ignore_comment(&mut self, source: &[u8]) -> Result<(), TokenizerError> {
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
                return Err(TokenizerError {
                    desc: format!("unterminated comment ({}:{})", start.line, start.column),
                });
            }
        }

        self.next_line();

        Ok(())
    }

    fn ignore_multiline_comment(&mut self, source: &[u8]) -> Result<(), TokenizerError> {
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
                return Err(TokenizerError {
                    desc: format!("unterminated comment ({}:{})", start.line, start.column),
                });
            }
        }

        Ok(())
    }

    fn comment(&mut self, source: &[u8]) -> Result<(), TokenizerError> {
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
            Some(c) => Err(TokenizerError {
                desc: format!(
                    "expected '/' or '*' not '{}' ({}:{})",
                    *c as char,
                    self.line(),
                    self.column()
                ),
            }),

            None => Err(TokenizerError {
                desc: format!(
                    "expected '/' or '*' but no bytes left ({}:{})",
                    start.line, start.column
                ),
            }),
        }
    }

    pub fn new() -> Self {
        Tokenizer {
            index: 0,
            column: 1,
            line: 1,
        }
    }

    pub fn tokenize<'a>(&mut self, source: &'a [u8]) -> Result<Vec<TokenKind<'a>>, TokenizerError> {
        let mut tokens = Vec::with_capacity(std::mem::size_of_val(source));

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

                // Keyword
                b'a'..=b'z' | b'A'..=b'Z' => match self.keyword(source) {
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

                // Int | Float
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
                    return Err(TokenizerError {
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
