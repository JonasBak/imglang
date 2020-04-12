use std::fs;

#[derive(Debug)]
pub struct Lexer {
    string: String,
}

pub type LexerResult<T> = Result<T, LexerError>;

#[derive(Debug)]
pub enum LexerError {
    Parse(usize),
    Unescaped(usize),
    File,
}

#[derive(Debug)]
pub enum TokenType {
    // Characters
    LeftPar,
    RightPar,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // Two/prefixes
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Lexer {
    pub fn new(string: String) -> Lexer {
        Lexer { string }
    }
    pub fn new_from_file(filename: &str) -> LexerResult<Lexer> {
        let string = fs::read_to_string(filename).or(Err(LexerError::File))?;
        Ok(Lexer::new(string))
    }
    pub fn parse(self) -> LexerResult<Vec<TokenType>> {
        let mut tokens = vec![];
        let mut chars = self.string.chars().enumerate().peekable();
        loop {
            let (i, c) = match chars.next() {
                Some(a) => a,
                None => {
                    break;
                }
            };
            let token = match c {
                '(' => TokenType::LeftPar,
                ')' => TokenType::RightPar,
                '[' => TokenType::LeftBrace,
                ']' => TokenType::RightBrace,
                ',' => TokenType::Comma,
                '.' => TokenType::Dot,
                '-' => TokenType::Minus,
                '+' => TokenType::Plus,
                ';' => TokenType::Semicolon,
                '*' => TokenType::Star,
                '/' => {
                    if chars.peek().map(|(_, cl)| cl == &'/').unwrap_or(false) {
                        loop {
                            if chars.next().map(|(_, cmt)| cmt == '\n').unwrap_or(true) {
                                break;
                            }
                        }
                        continue;
                    } else {
                        TokenType::Slash
                    }
                }

                '!' => {
                    if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                        chars.next();
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    }
                }
                '=' => {
                    if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                        chars.next();
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    }
                }
                '>' => {
                    if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                        chars.next();
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    }
                }
                '<' => {
                    if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                        chars.next();
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    }
                }

                ' ' | '\t' | '\r' | '\n' => {
                    continue;
                }

                '"' => {
                    let mut literal = vec![];
                    loop {
                        let (_, l) = chars.next().ok_or(LexerError::Unescaped(i))?;
                        if l == '"' {
                            break;
                        }
                        literal.push(l);
                    }
                    TokenType::String(literal.into_iter().collect())
                }
                '0'..='9' => {
                    let mut literal = vec![c];
                    loop {
                        if let Some((_, l)) = chars.peek() {
                            if !l.is_numeric() && l != &'.' {
                                break;
                            }
                            let (_, l) = chars.next().unwrap();
                            literal.push(l);
                        } else {
                            break;
                        }
                    }
                    TokenType::Number(
                        literal
                            .into_iter()
                            .collect::<String>()
                            .parse()
                            .or(Err(LexerError::Parse(i)))?,
                    )
                }

                _ => {
                    if c.is_alphanumeric() {
                        let mut literal = vec![c];
                        loop {
                            if let Some((_, l)) = chars.peek() {
                                if !l.is_alphanumeric() {
                                    break;
                                }
                                let (_, l) = chars.next().unwrap();
                                literal.push(l);
                            } else {
                                break;
                            }
                        }
                        TokenType::Identifier(literal.into_iter().collect::<String>())
                    } else {
                        return Err(LexerError::Parse(i));
                    }
                }
            };
            tokens.push(token);
        }
        tokens = tokens
            .into_iter()
            .map(|t| match t {
                TokenType::Identifier(val) => match val.as_str() {
                    "and" => TokenType::And,
                    "class" => TokenType::Class,
                    "else" => TokenType::Else,
                    "false" => TokenType::False,
                    "fun" => TokenType::Fun,
                    "for" => TokenType::For,
                    "if" => TokenType::If,
                    "nil" => TokenType::Nil,
                    "or" => TokenType::Or,
                    "print" => TokenType::Print,
                    "return" => TokenType::Return,
                    "super" => TokenType::Super,
                    "this" => TokenType::This,
                    "true" => TokenType::True,
                    "var" => TokenType::Var,
                    "while" => TokenType::While,
                    _ => TokenType::Identifier(val),
                },
                _ => t,
            })
            .collect();

        tokens.push(TokenType::Eof);
        return Ok(tokens);
    }
}
