use std;
use std::error::Error;

#[derive(Debug)]
pub enum Token<'a> {
    Keyword(&'a str),
    Str(&'a str),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    TableTerm,
    TablePrec,
    ListPrec,
    ListTerm,
}

#[derive(Debug)]
pub struct TokenizerError {
    pub description: String,
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TokenizerError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

fn ascii_to_float(_bytes: &[u8]) -> Result<f64, TokenizerError> {
    let float_str = std::str::from_utf8(_bytes);

    match float_str {
        Ok(str) => match str.parse::<f64>() {
            Ok(v) => Ok(v),
            Err(e) => Err(TokenizerError {
                description: format!("error converting to float: {:?} value: {str}", e),
            }),
        },
        Err(e) => Err(TokenizerError {
            description: format!("error converting to float: {:?}", e),
        }),
    }
}

fn ascii_to_int(_bytes: &[u8]) -> Result<i64, TokenizerError> {
    let float_str = std::str::from_utf8(_bytes);

    match float_str {
        Ok(str) => match str.parse::<i64>() {
            Ok(v) => Ok(v),
            Err(e) => Err(TokenizerError {
                description: format!("error converting to int: {:?} value: {str}", e),
            }),
        },
        Err(e) => Err(TokenizerError {
            description: format!("error converting to int: {:?}", e),
        }),
    }
}

fn unescape_bytes(_bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(_bytes.len());
    let mut chars = _bytes.iter().peekable();
    while let Some(&c) = chars.next() {
        match c {
            b'\\' => {
                if let Some(&&b) = chars.peek() {
                    chars.next();

                    match b {
                        b'"' => output.push(b'"'),
                        b'\'' => output.push(b'\''),
                        b'\\' => output.push(b'\\'),
                        b'n' => output.push(b'\n'),
                        b'r' => output.push(b'\r'),
                        b't' => output.push(b'\t'),
                        b'0' => output.push(b'\0'),
                        _ => output.push(b'\\'),
                    }
                }
            }
            _ => output.push(c),
        }
    }

    output
}

pub struct Tokenizer<'a> {
    inp: &'a [u8],
    start: usize,
    line: usize,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(inp: &'a [u8]) -> Self {
        Tokenizer {
            inp,
            start: 0,
            pos: 0,
            line: 0,
        }
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.pos += 1
    }

    #[inline(always)]
    fn string(&mut self) -> Result<Token<'a>, TokenizerError> {
        self.advance(); // skip opening double quote
        self.start = self.pos;

        loop {
            if let Some(&b) = self.inp.get(self.pos) {
                if b == b'"' {
                    break;
                }

                if b != b'\n' {
                    self.advance();
                } else {
                    return Err(TokenizerError {
                        description: "cannot use newline character in strings".to_string(),
                    });
                }
            } else {
                return Err(TokenizerError {
                    description: "unterminated string".to_string(),
                });
            }
        }

        let to_resolve = &self.inp[self.start..self.pos];

        self.advance(); // skip closing double quote

        if to_resolve.contains(&b'\\') {
            match String::from_utf8(unescape_bytes(to_resolve)) {
                Ok(s) => Ok(Token::String(s)),
                Err(e) => Err(TokenizerError {
                    description: format!("could not resolve string caused by: {:?}", e),
                }),
            }
        } else {
            match std::str::from_utf8(to_resolve) {
                Ok(s) => Ok(Token::Str(s)),
                Err(e) => Err(TokenizerError {
                    description: format!("could not resolve str caused by: {:?}", e),
                }),
            }
        }
    }

    #[inline(always)]
    fn number(&mut self) -> Result<Token<'a>, TokenizerError> {
        let mut point = false;
        let mut zero = false;

        if let Some(b) = self.inp.get(self.pos) {
            if *b == b'0' {
                zero = true;
            }
        }

        // let start_index = self.pos;

        self.start = self.pos;
        self.advance();

        while let Some(b) = self.inp.get(self.pos) {
            if b.is_ascii_digit() && !zero {
                self.advance();
            } else if *b == b'.' && !point {
                point = true;
                zero = false;

                self.advance();

                if let Some(b) = self.inp.get(self.pos) {
                    if !b.is_ascii_digit() {
                        return Err(TokenizerError {
                            description: format!(
                                "decimal point must be followed with a digit, not {}",
                                b
                            ),
                        });
                    }
                } else {
                    return Err(TokenizerError {
                        description:
                            "decimal point must be followed with a digit, but no bytes left"
                                .to_string(),
                    });
                }
            } else {
                break;
            }
        }

        let to_resolve = &self.inp[self.start..self.pos];

        if point {
            match ascii_to_float(to_resolve) {
                Ok(v) => Ok(Token::Float(v)),
                Err(e) => Err(e),
            }
        } else {
            match ascii_to_int(to_resolve) {
                Ok(v) => Ok(Token::Integer(v)),
                Err(e) => Err(e),
            }
        }
    }

    #[inline(always)]
    fn keyword(&mut self) -> Result<Token<'a>, TokenizerError> {
        self.start = self.pos;

        while let Some(b) = self.inp.get(self.pos) {
            if !(b.is_ascii_alphanumeric() || *b == b'_') {
                break;
            }

            self.advance();
        }

        match std::str::from_utf8(&self.inp[self.start..self.pos]) {
            Ok(kw) => match kw {
                // reserved
                "true" => Ok(Token::Boolean(true)),
                "false" => Ok(Token::Boolean(false)),

                _ => Ok(Token::Keyword(kw)),
            },

            Err(e) => Err(TokenizerError {
                description: format!("failed to resolve keyword caused by {}", e),
            }),
        }
    }

    #[inline(always)]
    pub fn create_tokens(&mut self) -> Result<Vec<Token<'a>>, TokenizerError> {
        let mut tokens = Vec::new();

        while let Some(b) = self.inp.get(self.pos) {
            match b {
                b'{' => {
                    tokens.push(Token::TablePrec);
                    self.advance();
                }
                b'}' => {
                    tokens.push(Token::TableTerm);
                    self.advance();
                }
                b'[' => {
                    tokens.push(Token::ListPrec);
                    self.advance();
                }
                b']' => {
                    tokens.push(Token::ListTerm);
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'\r' | b'\t' | b' ' => self.advance(), // skip whitespaces,

                // Keyword
                b'a'..=b'z' | b'A'..=b'Z' => match self.keyword() {
                    Ok(kw) => tokens.push(kw),
                    Err(e) => return Err(e),
                },

                // String
                b'"' => match self.string() {
                    Ok(s) => tokens.push(s),
                    Err(e) => return Err(e),
                },

                // Integer | Float
                b'0'..=b'9' | b'+' | b'-' => match self.number() {
                    Ok(num) => tokens.push(num),
                    Err(e) => return Err(e),
                },
                _ => {
                    return Err(TokenizerError {
                        description: format!(
                            "unrecognized character '{}' line: {}",
                            *b as char, self.line
                        ),
                    })
                }
            }
        }

        Ok(tokens)
    }
}
