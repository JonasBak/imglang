use imglang::*;

#[test]
fn unescaped_string_error() {
    let res = parse_string(&"var noe = \"hehe".to_string());
    assert_eq!(res.unwrap_err(), LexerError::Unescaped(10));
}

#[test]
fn illegal_character_error() {
    let res = parse_string(&"var noe @ =  \"hehe\"".to_string());
    assert_eq!(res.unwrap_err(), LexerError::Parse(8));
}

#[test]
fn empty_string() {
    let mut tokens = parse_string(&"".to_string()).unwrap().into_iter();
    assert_eq!(
        tokens.next(),
        Some(Token {
            start: 0,
            end: 0,
            t: TokenType::Eof
        })
    );
    assert_eq!(tokens.next(), None);
}

#[test]
fn whitespace_ignored() {
    let mut tokens = parse_string(&"    \n\n  \r\t \t".to_string())
        .unwrap()
        .into_iter();
    assert_eq!(
        tokens.next(),
        Some(Token {
            start: 12,
            end: 12,
            t: TokenType::Eof
        })
    );
    assert_eq!(tokens.next(), None);
}

#[test]
fn signle_characters() {
    let mut tokens = parse_string(&"()[],.-+;/*".to_string())
        .unwrap()
        .into_iter();
    let expected = vec![
        Some(Token {
            start: 0,
            end: 1,
            t: TokenType::LeftPar,
        }),
        Some(Token {
            start: 1,
            end: 2,
            t: TokenType::RightPar,
        }),
        Some(Token {
            start: 2,
            end: 3,
            t: TokenType::LeftBrace,
        }),
        Some(Token {
            start: 3,
            end: 4,
            t: TokenType::RightBrace,
        }),
        Some(Token {
            start: 4,
            end: 5,
            t: TokenType::Comma,
        }),
        Some(Token {
            start: 5,
            end: 6,
            t: TokenType::Dot,
        }),
        Some(Token {
            start: 6,
            end: 7,
            t: TokenType::Minus,
        }),
        Some(Token {
            start: 7,
            end: 8,
            t: TokenType::Plus,
        }),
        Some(Token {
            start: 8,
            end: 9,
            t: TokenType::Semicolon,
        }),
        Some(Token {
            start: 9,
            end: 10,
            t: TokenType::Slash,
        }),
        Some(Token {
            start: 10,
            end: 11,
            t: TokenType::Star,
        }),
        Some(Token {
            start: 11,
            end: 11,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}

#[test]
fn prefix_characters() {
    let mut tokens = parse_string(&"! != = == > >= < <=".to_string())
        .unwrap()
        .into_iter();
    let expected = vec![
        Some(Token {
            start: 0,
            end: 1,
            t: TokenType::Bang,
        }),
        Some(Token {
            start: 2,
            end: 4,
            t: TokenType::BangEqual,
        }),
        Some(Token {
            start: 5,
            end: 6,
            t: TokenType::Equal,
        }),
        Some(Token {
            start: 7,
            end: 9,
            t: TokenType::EqualEqual,
        }),
        Some(Token {
            start: 10,
            end: 11,
            t: TokenType::Greater,
        }),
        Some(Token {
            start: 12,
            end: 14,
            t: TokenType::GreaterEqual,
        }),
        Some(Token {
            start: 15,
            end: 16,
            t: TokenType::Lesser,
        }),
        Some(Token {
            start: 17,
            end: 19,
            t: TokenType::LesserEqual,
        }),
        Some(Token {
            start: 19,
            end: 19,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}

#[test]
fn comments_ignored_to_newline() {
    let mut tokens = parse_string(&"// test\n. // test 123".to_string())
        .unwrap()
        .into_iter();
    let expected = vec![
        Some(Token {
            start: 8,
            end: 9,
            t: TokenType::Dot,
        }),
        Some(Token {
            start: 21,
            end: 21,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}

#[test]
fn literals() {
    let mut tokens = parse_string(&"identifier \"string\" 123.456".to_string())
        .unwrap()
        .into_iter();
    let expected = vec![
        Some(Token {
            start: 0,
            end: 10,
            t: TokenType::Identifier("identifier".to_string()),
        }),
        Some(Token {
            start: 11,
            end: 19,
            t: TokenType::String("string".to_string()),
        }),
        Some(Token {
            start: 20,
            end: 27,
            t: TokenType::Number(123.456),
        }),
        Some(Token {
            start: 27,
            end: 27,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}

#[test]
fn keywords() {
    let mut tokens = parse_string(
        &"and class else false fun for if nil or print return super this true var while"
            .to_string(),
    )
    .unwrap()
    .into_iter();
    let expected = vec![
        Some(Token {
            start: 0,
            end: 3,
            t: TokenType::And,
        }),
        Some(Token {
            start: 4,
            end: 9,
            t: TokenType::Class,
        }),
        Some(Token {
            start: 10,
            end: 14,
            t: TokenType::Else,
        }),
        Some(Token {
            start: 15,
            end: 20,
            t: TokenType::False,
        }),
        Some(Token {
            start: 21,
            end: 24,
            t: TokenType::Fun,
        }),
        Some(Token {
            start: 25,
            end: 28,
            t: TokenType::For,
        }),
        Some(Token {
            start: 29,
            end: 31,
            t: TokenType::If,
        }),
        Some(Token {
            start: 32,
            end: 35,
            t: TokenType::Nil,
        }),
        Some(Token {
            start: 36,
            end: 38,
            t: TokenType::Or,
        }),
        Some(Token {
            start: 39,
            end: 44,
            t: TokenType::Print,
        }),
        Some(Token {
            start: 45,
            end: 51,
            t: TokenType::Return,
        }),
        Some(Token {
            start: 52,
            end: 57,
            t: TokenType::Super,
        }),
        Some(Token {
            start: 58,
            end: 62,
            t: TokenType::This,
        }),
        Some(Token {
            start: 63,
            end: 67,
            t: TokenType::True,
        }),
        Some(Token {
            start: 68,
            end: 71,
            t: TokenType::Var,
        }),
        Some(Token {
            start: 72,
            end: 77,
            t: TokenType::While,
        }),
        Some(Token {
            start: 77,
            end: 77,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}

#[test]
fn identifier_with_keyword_prefix() {
    let mut tokens = parse_string(&"format variable iff".to_string())
        .unwrap()
        .into_iter();
    let expected = vec![
        Some(Token {
            start: 0,
            end: 6,
            t: TokenType::Identifier("format".to_string()),
        }),
        Some(Token {
            start: 7,
            end: 15,
            t: TokenType::Identifier("variable".to_string()),
        }),
        Some(Token {
            start: 16,
            end: 19,
            t: TokenType::Identifier("iff".to_string()),
        }),
        Some(Token {
            start: 19,
            end: 19,
            t: TokenType::Eof,
        }),
        None,
    ];
    for token in expected.into_iter() {
        assert_eq!(token, tokens.next());
    }
}
