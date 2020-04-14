use super::*;

pub fn print_parser_err(source: &String, error: ParserError) {
    let mut source = source.clone();
    if let ParserError::UnexpectedToken(token) = error {
        source.insert_str(token.end, "<<<");
        source.insert_str(token.start, ">>>");
        println!("{}", source);
        println!("Error: Unexpected token {:?}", token.t);
    } else {
        println!("Error: {:?}", error);
    }
}

pub fn print_lexer_err(source: &String, error: LexerError) {
    let mut source = source.clone();
    match error {
        LexerError::Parse(i) => source.insert_str(i, ">>>"),
        LexerError::Unescaped(i) => source.insert_str(i, ">>>"),
    }
    println!("{}", source);
    match error {
        LexerError::Parse(i) => {
            println!("Error: Could not parse character at position {}", i);
        }
        LexerError::Unescaped(i) => {
            println!("Error: Unescaped string starting at {}", i);
        }
    }
}
