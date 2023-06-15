use mcl::{Parser, Tokenizer};

#[test]
fn test_mcl() {
    let file = std::fs::read("./tests/sample.mcl").unwrap();

    let mut tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(&file).unwrap();

    let mut parser = Parser::new();
    let output = parser.parse(&tokens).unwrap();

    println!("{:?}", output);
}
