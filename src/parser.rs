use super::*;
use std::iter::Peekable;

trait TokenIterator<'a>: Iterator<Item = &'a Token> {
    fn peek(&mut self) -> Option<&'a Token>;
    fn peek_t(&mut self) -> Option<&'a TokenType> {
        self.peek().map(|t| &t.t)
    }
    fn next_t(&mut self) -> Option<&'a TokenType> {
        self.next().map(|t| &t.t)
    }
    fn check(&mut self, ped: &dyn Fn(&TokenType) -> bool) -> bool {
        self.peek_t().map(|t| ped(t)).unwrap_or(false)
    }
    fn unexpected(&mut self) -> ParserError {
        match self.next() {
            Some(token) => ParserError::UnexpectedToken(token.clone()),
            None => ParserError::TODO,
        }
    }
}

fn map_when<T>(
    tokens: &mut dyn TokenIterator,
    ped: &dyn Fn(&TokenType) -> Option<T>,
) -> ParserResult<T>
where
    T: Clone,
{
    match tokens.peek_t().map(|t| ped(t)).flatten() {
        Some(i) => {
            tokens.next();
            Ok(i.clone())
        }
        _ => return Err(tokens.unexpected()),
    }
}

impl<'a, I> TokenIterator<'a> for Peekable<I>
where
    I: Iterator<Item = &'a Token>,
{
    fn peek(&mut self) -> Option<&'a Token> {
        self.peek().map(|t| *t)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    TODO,
}

#[derive(Debug, PartialEq)]
pub enum Ast {
    // Misc
    Program(Vec<Box<Ast>>),
    Decl(String, Box<Ast>),
    Assign(String, Box<Ast>),
    Print(Box<Ast>),
    Block(Vec<Box<Ast>>),

    // Flow
    While { condition: Box<Ast>, body: Box<Ast> },

    // primary
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Identifier(String),

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
    NotEq(Box<Ast>, Box<Ast>),
}

pub fn parse_program(tokens: Vec<Token>) -> ParserResult<Box<Ast>> {
    let mut tokens = tokens.iter().peekable();
    let mut prog = vec![];
    while !tokens.check(&|t| t == &TokenType::Eof) {
        prog.push(parse_declaration(&mut tokens)?);
    }
    tokens.next();
    Ok(Box::new(Ast::Program(prog)))
}

fn parse_primary(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    match tokens.peek_t() {
        Some(TokenType::Number(n)) => {
            tokens.next();
            Ok(Box::new(Ast::Number(*n)))
        }
        Some(TokenType::String(string)) => {
            tokens.next();
            Ok(Box::new(Ast::String(string.clone())))
        }
        Some(TokenType::False) => {
            tokens.next();
            Ok(Box::new(Ast::Bool(false)))
        }
        Some(TokenType::True) => {
            tokens.next();
            Ok(Box::new(Ast::Bool(true)))
        }
        Some(TokenType::Nil) => {
            tokens.next();
            Ok(Box::new(Ast::Nil))
        }
        Some(TokenType::Identifier(s)) => {
            tokens.next();
            Ok(Box::new(Ast::Identifier(s.clone())))
        }
        Some(TokenType::LeftPar) => {
            tokens.next();
            let node = parse_expression(tokens)?;
            if !tokens.check(&|t| t == &TokenType::RightPar) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            Ok(node)
        }
        _ => Err(tokens.unexpected()),
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

    while tokens.check(&|t| match t {
        TokenType::Slash | TokenType::Star => true,
        _ => false,
    }) {
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

    while tokens.check(&|t| match t {
        TokenType::Minus | TokenType::Plus => true,
        _ => false,
    }) {
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

    while tokens.check(&|t| match t {
        TokenType::Greater
        | TokenType::GreaterEqual
        | TokenType::Lesser
        | TokenType::LesserEqual => true,
        _ => false,
    }) {
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

    while tokens.check(&|t| match t {
        TokenType::BangEqual | TokenType::EqualEqual => true,
        _ => false,
    }) {
        eq = match tokens.next().unwrap().t {
            TokenType::EqualEqual => Box::new(Ast::Eq(eq, parse_comparison(tokens)?)),
            TokenType::BangEqual => Box::new(Ast::NotEq(eq, parse_comparison(tokens)?)),
            _ => panic!(),
        }
    }
    Ok(eq)
}

fn parse_assignment(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    let left = parse_equality(tokens)?;
    match tokens.peek_t() {
        Some(TokenType::Equal) => match *left {
            Ast::Identifier(identifier) => {
                tokens.next();
                let value = parse_assignment(tokens)?;
                Ok(Box::new(Ast::Assign(identifier, value)))
            }
            _ => Err(tokens.unexpected()),
        },
        _ => Ok(left),
    }
}

fn parse_expression(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    parse_assignment(tokens)
}

fn parse_statement(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    match tokens.peek_t() {
        Some(TokenType::Print) => {
            tokens.next();
            let print = parse_expression(tokens)?;
            if !tokens.check(&|t| t == &TokenType::Semicolon) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            Ok(Box::new(Ast::Print(print)))
        }
        Some(TokenType::LeftBrace) => {
            tokens.next();
            let mut block = vec![];
            while !tokens.check(&|t| t == &TokenType::RightBrace) {
                block.push(parse_declaration(tokens)?);
            }
            tokens.next();
            Ok(Box::new(Ast::Block(block)))
        }
        Some(TokenType::While) => {
            tokens.next();
            if !tokens.check(&|t| t == &TokenType::LeftPar) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            let condition = parse_expression(tokens)?;
            if !tokens.check(&|t| t == &TokenType::RightPar) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            let body = parse_statement(tokens)?;
            Ok(Box::new(Ast::While { condition, body }))
        }
        _ => {
            let stmt = parse_expression(tokens)?;
            if !tokens.check(&|t| t == &TokenType::Semicolon) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            Ok(stmt)
        }
    }
}

fn parse_declaration(tokens: &mut dyn TokenIterator) -> ParserResult<Box<Ast>> {
    match tokens.peek_t() {
        Some(TokenType::Var) => {
            tokens.next();
            let identifier = map_when(tokens, &|t| match t {
                TokenType::Identifier(s) => Some(s.clone()),
                _ => None,
            })?;
            if !tokens.check(&|t| t == &TokenType::Equal) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            let expr = parse_expression(tokens)?;
            if !tokens.check(&|t| t == &TokenType::Semicolon) {
                return Err(tokens.unexpected());
            }
            tokens.next();
            Ok(Box::new(Ast::Decl(identifier, expr)))
        }
        _ => parse_statement(tokens),
    }
}
