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
    TODO,
    UnexpectedToken(Token),
    // For when there is a None where there shouldn't be one
    LogicError,
}

pub enum Unary {
    Unary(Box<Unary>),
    Primary(Option<Box<Expression>>),
}

pub struct Multiplication(Box<Unary>, Option<Box<Unary>>);

pub struct Addition(Box<Multiplication>, Option<Box<Multiplication>>);

pub struct Comparison(Box<Addition>, Option<Box<Addition>>);

pub struct Equality(Box<Comparison>, Option<Box<Comparison>>);

pub struct Expression(Box<Equality>);

pub fn parse_tokens(tokens: Vec<Token>) -> ParserResult<()> {
    let mut tokens = tokens.iter().peekable();
    parse_expression(&mut tokens)?;
    Ok(())
}

fn parse_primary(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    match tokens.peek_t().ok_or(ParserError::LogicError)? {
        TokenType::Number(_)
        | TokenType::String(_)
        | TokenType::False
        | TokenType::True
        | TokenType::Nil => {
            tokens.next();
            Ok(())
        }
        TokenType::LeftPar => {
            parse_expression(tokens)?;
            match tokens.peek_t() {
                Some(TokenType::RightPar) => {
                    tokens.next();
                    Ok(())
                }
                _ => Err(ParserError::UnexpectedToken(
                    tokens.next().ok_or(ParserError::LogicError)?.clone(),
                )),
            }
        }

        _ => Err(ParserError::UnexpectedToken(tokens.next().unwrap().clone())),
    }
}

fn parse_unary(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    match tokens.peek_t() {
        Some(TokenType::Bang) | Some(TokenType::Minus) => {
            tokens.next();
            parse_unary(tokens)?;
        }
        _ => {
            parse_primary(tokens)?;
        }
    }
    Ok(())
}

fn parse_multiplication(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    parse_unary(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::Slash | TokenType::Star => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        tokens.next();
        parse_unary(tokens)?;
    }
    Ok(())
}

fn parse_addition(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    parse_multiplication(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::Minus | TokenType::Plus => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        tokens.next();
        parse_multiplication(tokens)?;
    }
    Ok(())
}

fn parse_comparison(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    parse_addition(tokens)?;

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
        tokens.next();
        parse_addition(tokens)?;
    }
    Ok(())
}

fn parse_equality(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    parse_comparison(tokens)?;

    while tokens
        .peek_t()
        .map(|t| match t {
            TokenType::BangEqual | TokenType::EqualEqual => true,
            _ => false,
        })
        .unwrap_or(false)
    {
        tokens.next();
        parse_comparison(tokens)?;
    }
    Ok(())
}

fn parse_expression(tokens: &mut dyn TokenIterator) -> ParserResult<()> {
    parse_equality(tokens)?;
    Ok(())
}
