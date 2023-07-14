use mcl::{Lexer, Parser};

#[test]
fn test_mcl() {
    let file = std::fs::read("./tests/sample.mcl").unwrap();

    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(&file).unwrap();

    let mut parser = Parser::new();
    let output = parser.parse(&tokens).unwrap();

    println!("{:?}", output);
}
