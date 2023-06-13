use mcl::parser;
use mcl::tokenizer;

fn main() {
    let start = std::time::Instant::now();
    let file = std::fs::read("./bench/small-sample3.mcl").unwrap();

    let mut tokenizer = tokenizer::Tokenizer::new();
    let tokens = tokenizer.tokenize(&file).unwrap();

    let mut parser = parser::Parser::new();
    let parsed = parser.parse(&tokens).unwrap();

    let elapsed = start.elapsed();

    println!("{:?}", parsed);
    println!("{:?}", elapsed);
}

// SHOULD HAVE LOC
// implement structs instead
