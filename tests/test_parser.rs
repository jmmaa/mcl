use mcl::parser;
use mcl::tokenizer;

#[test]
fn test_mcl() {
    let file = std::fs::read("./sample.mcl").unwrap();

    let mut tokenizer = tokenizer::Tokenizer::new();
    let tokens = tokenizer.tokenize(&file).unwrap();

    let mut parser = parser::Parser::new();
    parser.parse(&tokens).unwrap();
}
