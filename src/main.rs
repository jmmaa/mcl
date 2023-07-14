use mcl::{Lexer, Parser};

use std::time::Instant;

fn main() {
    let start = Instant::now();

    let file_start = Instant::now();

    let file = std::fs::read("./bench/big-sample2.mcl").unwrap();

    let file_read_time = file_start.elapsed();

    let lexer_start = Instant::now();

    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(&file).unwrap();

    let tokenizing_time = lexer_start.elapsed();

    let parser_start = Instant::now();

    let mut parser = Parser::new();
    let output = parser.parse(&tokens).unwrap();

    let parsing_time = parser_start.elapsed();

    let total = start.elapsed();

    println!(
        r#"
    file read:      {file_read_time:.2?} seconds\n
    tokenization:   {tokenizing_time:.2?} seconds\n
    parsing:        {parsing_time:.2?} seconds\n
    "#
    );

    println!("{:?}\nfinished {:.2?} seconds", output, total);
}

// go with &str appraoch on lexer
