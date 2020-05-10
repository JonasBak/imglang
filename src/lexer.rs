pub type LexerResult<T> = Result<T, LexerError>;
#[derive(Debug, PartialEq)]
pub enum LexerError {
    Parse(usize),
    Unescaped(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Characters
    LeftPar,
    RightPar,
    LeftBrace,
    RightBrace,
    LeftSquare,
    RightSquare,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bar,

    // Two/prefixes
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,

    // Literals
    Identifier(String),
    String(String),
    Float(f64),

    // Keywords
    And,
    Else,
    False,
    Fun,
    For,
    If,
    Or,
    Print,
    Return,
    True,
    Var,
    While,
    Enum,

    TypeFloat,
    TypeBool,
    TypeNil,
    TypeString,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub t: TokenType,
}

pub struct Lexer {
    tokens: Vec<Token>,
    current: usize,
}

impl Lexer {
    pub fn new(string: &String) -> LexerResult<Lexer> {
        Ok(Lexer {
            tokens: scan(string)?,
            current: 0,
        })
    }
    pub fn prev(&self) -> Option<Token> {
        self.tokens.get(self.current - 1).cloned()
    }
    pub fn prev_t(&self) -> Option<TokenType> {
        self.prev().map(|t| t.t)
    }
    pub fn current(&self) -> Token {
        self.tokens[self.current].clone()
    }
    pub fn current_t(&self) -> TokenType {
        self.tokens[self.current].t.clone()
    }
    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current + 1).cloned()
    }
    pub fn next(&mut self) -> Option<Token> {
        match self.peek() {
            Some(t) => {
                self.current += 1;
                Some(t)
            }
            None => None,
        }
    }
}

pub fn scan(string: &String) -> LexerResult<Vec<Token>> {
    let mut tokens = vec![];
    let mut chars = string.chars().enumerate().peekable();
    loop {
        let (i, c) = match chars.next() {
            Some(a) => a,
            None => {
                break;
            }
        };
        let (start, end, t) = match c {
            '(' => (i, i + 1, TokenType::LeftPar),
            ')' => (i, i + 1, TokenType::RightPar),
            '{' => (i, i + 1, TokenType::LeftBrace),
            '}' => (i, i + 1, TokenType::RightBrace),
            '[' => (i, i + 1, TokenType::LeftSquare),
            ']' => (i, i + 1, TokenType::RightSquare),
            ',' => (i, i + 1, TokenType::Comma),
            '.' => (i, i + 1, TokenType::Dot),
            '-' => (i, i + 1, TokenType::Minus),
            '+' => (i, i + 1, TokenType::Plus),
            ';' => (i, i + 1, TokenType::Semicolon),
            '*' => (i, i + 1, TokenType::Star),
            '|' => (i, i + 1, TokenType::Bar),
            '/' => {
                if chars.peek().map(|(_, cl)| cl == &'/').unwrap_or(false) {
                    loop {
                        if chars.next().map(|(_, cmt)| cmt == '\n').unwrap_or(true) {
                            break;
                        }
                    }
                    continue;
                } else {
                    (i, i + 1, TokenType::Slash)
                }
            }

            '!' => {
                if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                    chars.next();
                    (i, i + 2, TokenType::BangEqual)
                } else {
                    (i, i + 1, TokenType::Bang)
                }
            }
            '=' => {
                if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                    chars.next();
                    (i, i + 2, TokenType::EqualEqual)
                } else {
                    (i, i + 1, TokenType::Equal)
                }
            }
            '>' => {
                if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                    chars.next();
                    (i, i + 2, TokenType::GreaterEqual)
                } else {
                    (i, i + 1, TokenType::Greater)
                }
            }
            '<' => {
                if chars.peek().map(|(_, cl)| cl == &'=').unwrap_or(false) {
                    chars.next();
                    (i, i + 2, TokenType::LesserEqual)
                } else {
                    (i, i + 1, TokenType::Lesser)
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
                (
                    i,
                    i + literal.len() + 2,
                    TokenType::String(literal.into_iter().collect()),
                )
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
                (
                    i,
                    i + literal.len(),
                    TokenType::Float(
                        literal
                            .into_iter()
                            .collect::<String>()
                            .parse()
                            .or(Err(LexerError::Parse(i)))?,
                    ),
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
                    (
                        i,
                        i + literal.len(),
                        TokenType::Identifier(literal.into_iter().collect::<String>()),
                    )
                } else {
                    return Err(LexerError::Parse(i));
                }
            }
        };
        tokens.push(Token { start, end, t });
    }
    tokens = tokens
        .into_iter()
        .map(|mut t| {
            t.t = match t.t {
                TokenType::Identifier(val) => match val.as_str() {
                    "and" => TokenType::And,
                    "else" => TokenType::Else,
                    "false" => TokenType::False,
                    "fun" => TokenType::Fun,
                    "for" => TokenType::For,
                    "if" => TokenType::If,
                    "or" => TokenType::Or,
                    "print" => TokenType::Print,
                    "return" => TokenType::Return,
                    "true" => TokenType::True,
                    "var" => TokenType::Var,
                    "while" => TokenType::While,
                    "enum" => TokenType::Enum,
                    "nil" => TokenType::TypeNil,
                    "float" => TokenType::TypeFloat,
                    "bool" => TokenType::TypeBool,
                    "str" => TokenType::TypeString,
                    _ => TokenType::Identifier(val),
                },
                _ => t.t,
            };
            t
        })
        .collect();

    tokens.push(Token {
        start: string.len(),
        end: string.len(),
        t: TokenType::Eof,
    });
    return Ok(tokens);
}
