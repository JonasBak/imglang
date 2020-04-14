use super::*;
use std::iter::Peekable;

trait TokenIterator<'a>: Iterator<Item = &'a Token> {
    fn peek(&mut self) -> Option<&'a Token>;
    fn peek_t(&mut self) -> Option<&'a TokenType>;
}

impl<'a, I> TokenIterator<'a> for Peekable<I>
where
    I: Iterator<Item = &'a Token>,
{
    fn peek(&mut self) -> Option<&'a Token> {
        self.peek().map(|t| *t)
    }
    fn peek_t(&mut self) -> Option<&'a TokenType> {
        self.peek().map(|t| &t.t)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    ExpectedToken,
    ExpectedEof,
}

pub fn parse_tokens(tokens: Vec<Token>) -> ParserResult<Box<Ast>> {
    let mut tokens = tokens.iter().peekable();
    let exp = parse_expression(&mut tokens)?;
    if tokens.next().map(|t| t.t != TokenType::Eof).unwrap_or(true) {
        return Err(ParserError::ExpectedEof);
    }
    Ok(exp)
}

#[derive(Debug)]
pub enum Ast {
    // primary
    Number(f64),
    String(String),
    False,
    True,
    Nil,

    // unary
    Bang(Box<Ast>),
    Negated(Box<Ast>),

    // binary
    Mul(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),

    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),

    G(Box<Ast>, Box<Ast>),
    GE(Box<Ast>, Box<Ast>),
    L(Box<Ast>, Box<Ast>),
    LE(Box<Ast>, Box<Ast>),

    Eq(Box<Ast>, Box<Ast>),
    Not(Box<Ast>, Box<Ast>),
}

fn parse_primary(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    match tokens.peek_t().ok_or(ParserError::ExpectedToken)? {
        TokenType::Number(n) => {
            tokens.next();
            Ok(Box::new(Ast::Number(*n)))
        }
        TokenType::String(string) => {
            tokens.next();
            Ok(Box::new(Ast::String(string.clone())))
        }
        TokenType::False => {
            tokens.next();
            Ok(Box::new(Ast::False))
        }
        TokenType::True => {
            tokens.next();
            Ok(Box::new(Ast::True))
        }
        TokenType::Nil => {
            tokens.next();
            Ok(Box::new(Ast::Nil))
        }
        TokenType::LeftPar => {
            tokens.next();
            let node = parse_expression(tokens)?;
            match tokens.peek_t() {
                Some(TokenType::RightPar) => {
                    tokens.next();
                    Ok(node)
                }
                _ => Err(ParserError::UnexpectedToken(tokens.next().unwrap().clone())),
            }
        }

        _ => Err(ParserError::UnexpectedToken(tokens.next().unwrap().clone())),
    }
}

fn parse_unary(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let unary = match tokens.peek_t() {
        Some(TokenType::Bang) => {
            tokens.next();
            Box::new(Ast::Bang(parse_unary(tokens)?))
        }
        Some(TokenType::Minus) => {
            tokens.next();
            Box::new(Ast::Negated(parse_unary(tokens)?))
        }
        _ => parse_primary(tokens)?,
    };
    Ok(unary)
}

fn parse_multiplication(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let mut mul = parse_unary(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::Slash | TokenType::Star => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        mul = match tokens.next().unwrap().t {
            TokenType::Star => Box::new(Ast::Mul(mul, parse_unary(tokens)?)),
            TokenType::Slash => Box::new(Ast::Div(mul, parse_unary(tokens)?)),
            _ => panic!(),
        }
    }
    Ok(mul)
}

fn parse_addition(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let mut add = parse_multiplication(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::Minus | TokenType::Plus => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        add = match tokens.next().unwrap().t {
            TokenType::Plus => Box::new(Ast::Add(add, parse_multiplication(tokens)?)),
            TokenType::Minus => Box::new(Ast::Sub(add, parse_multiplication(tokens)?)),
            _ => panic!(),
        }
    }
    Ok(add)
}

fn parse_comparison(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let mut com = parse_addition(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Lesser
            | TokenType::LesserEqual => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        com = match tokens.next().unwrap().t {
            TokenType::Greater => Box::new(Ast::G(com, parse_addition(tokens)?)),
            TokenType::GreaterEqual => Box::new(Ast::GE(com, parse_addition(tokens)?)),
            TokenType::Lesser => Box::new(Ast::L(com, parse_addition(tokens)?)),
            TokenType::LesserEqual => Box::new(Ast::LE(com, parse_addition(tokens)?)),
            _ => panic!(),
        }
    }
    Ok(com)
}

fn parse_equality(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let mut eq = parse_comparison(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::BangEqual | TokenType::EqualEqual => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        eq = match tokens.next().unwrap().t {
            TokenType::EqualEqual => Box::new(Ast::Eq(eq, parse_comparison(tokens)?)),
            TokenType::BangEqual => Box::new(Ast::Not(eq, parse_comparison(tokens)?)),
            _ => panic!(),
        }
    }
    Ok(eq)
}

fn parse_expression(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    parse_equality(tokens)
}
