pub use crate::prelude::*;

pub use serde_json;

pub mod error;
pub mod lexer;
pub mod parser;
pub mod prelude;
pub mod token;

use lexer::Lexer;
use parser::Parser;

pub fn from_str(v: &str) -> Result<serde_json::Value> {
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(v.as_bytes())?;

    let mut parser = Parser::new();
    let output = parser.parse(&tokens)?;

    Ok(output)
}

pub fn from_slice(v: &[u8]) -> Result<serde_json::Value> {
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(v)?;

    let mut parser = Parser::new();
    let output = parser.parse(&tokens)?;

    Ok(output)
}
