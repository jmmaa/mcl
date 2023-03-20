use mcl::parser::Parser;
use mcl::tokenizer::Tokenizer;

fn main() {
    let file = std::fs::read("./bench/small-sample3.mcl").unwrap();

    let start = std::time::Instant::now();

    // // FOR SIMPLE DEBUGGING
    // let parsed = tokenize("marky { gg 25.1 } ".as_bytes()).unwrap();
    // println!("{:?}", parsed);

    // FOR BENCHING
    // let tokens = tokenize(&file).unwrap();
    let tokens = Tokenizer::new(&file).create_tokens().unwrap();
    let parsed = Parser::new(&tokens).create_tree().unwrap();

    let elapsed = start.elapsed();

    println!("{:?}", parsed);

    println!("{:?}", elapsed);
}
