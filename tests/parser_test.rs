use imglang::*;

#[test]
fn empty_program() {
    let tokens = parse_string(&"".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(*ast.unwrap(), Ast::Program(vec![]));
}

#[test]
fn single_values() {
    let tokens = parse_string(&"1234;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Number(1234.0))])
    );
    let tokens = parse_string(&"\"abc\";".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::String("abc".to_string()))])
    );
    let tokens = parse_string(&"abc;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Identifier("abc".to_string()))])
    );
    let tokens = parse_string(&"true;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(*ast.unwrap(), Ast::Program(vec![Box::new(Ast::Bool(true))]));
    let tokens = parse_string(&"false;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Bool(false))])
    );
}

#[test]
fn unary() {
    let tokens = parse_string(&"-1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Negated(Box::new(Ast::Number(1.0))))])
    );
    let tokens = parse_string(&"!true;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Bang(Box::new(Ast::Bool(true))))])
    );
}

#[test]
fn binary() {
    let tokens = parse_string(&"1*1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Mul(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1/1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Div(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1+1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Add(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1-1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Sub(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1>1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::G(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1>=1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::GE(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1<1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::L(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1<=1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::LE(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1==1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Eq(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
    let tokens = parse_string(&"1!=1;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::NotEq(
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(1.0))
        ))])
    );
}

#[test]
fn multiple_lines() {
    let tokens = parse_string(&"1;\n2;\n3;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![
            Box::new(Ast::Number(1.0)),
            Box::new(Ast::Number(2.0)),
            Box::new(Ast::Number(3.0)),
        ])
    );
}

#[test]
fn unexpected_eof() {
    let tokens = parse_string(&"var a = ".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        ast.unwrap_err(),
        ParserError::UnexpectedToken(Token {
            start: 8,
            end: 8,
            t: TokenType::Eof
        })
    );
}

#[test]
fn unexpected_token() {
    let tokens = parse_string(&"var a = while;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        ast.unwrap_err(),
        ParserError::UnexpectedToken(Token {
            start: 8,
            end: 13,
            t: TokenType::While
        })
    );
}

#[test]
fn parse_while() {
    let tokens = parse_string(&"while (a < 10) print \"abc\";".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::While {
            condition: Box::new(Ast::L(
                Box::new(Ast::Identifier("a".to_string())),
                Box::new(Ast::Number(10.0))
            )),
            body: Box::new(Ast::Print(Box::new(Ast::String("abc".to_string()))))
        }),])
    );
}

#[test]
fn parse_declaration() {
    let tokens = parse_string(&"var abc = 123.3;".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Decl(
            "abc".to_string(),
            Box::new(Ast::Number(123.3))
        )),])
    );
}

#[test]
fn parse_nested_blocks() {
    let tokens = parse_string(&"{ { a = b; } }".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        *ast.unwrap(),
        Ast::Program(vec![Box::new(Ast::Block(vec![Box::new(Ast::Block(
            vec![Box::new(Ast::Assign(
                "a".to_string(),
                Box::new(Ast::Identifier("b".to_string()))
            ))]
        ))]))])
    );
}

#[test]
fn missing_closing_bracket() {
    let tokens = parse_string(&"{ { a = b; } ".to_string()).unwrap();
    let ast = parse_program(tokens);
    assert_eq!(
        ast.unwrap_err(),
        ParserError::UnexpectedToken(Token {
            start: 13,
            end: 13,
            t: TokenType::Eof
        })
    );
}
