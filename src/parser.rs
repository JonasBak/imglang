use super::*;
use std::iter::Peekable;

trait AstNode
where
    Self: std::marker::Sized,
{
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self>;
}

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
    // For when there is a None where there shouldn't be one, TODO
    LogicError,
}

pub fn parse_tokens(tokens: Vec<Token>) -> ParserResult<Expression> {
    let mut tokens = tokens.iter().peekable();
    Expression::parse(&mut tokens)
}

#[derive(Debug)]
pub enum Primary {
    Number(f64),
    String(String),
    False,
    True,
    Nil,
    GroupedExpr(Box<Expression>),
}
impl AstNode for Primary {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        match tokens.peek_t().ok_or(ParserError::LogicError)? {
            TokenType::Number(n) => {
                tokens.next();
                Ok(Primary::Number(*n))
            }
            TokenType::String(string) => {
                tokens.next();
                Ok(Primary::String(string.clone()))
            }
            TokenType::False => {
                tokens.next();
                Ok(Primary::False)
            }
            TokenType::True => {
                tokens.next();
                Ok(Primary::True)
            }
            TokenType::Nil => {
                tokens.next();
                Ok(Primary::Nil)
            }
            TokenType::LeftPar => {
                tokens.next();
                let node = Primary::GroupedExpr(Box::new(Expression::parse(tokens)?));
                match tokens.peek_t() {
                    Some(TokenType::RightPar) => {
                        tokens.next();
                        Ok(node)
                    }
                    _ => Err(ParserError::UnexpectedToken(
                        tokens.next().ok_or(ParserError::LogicError)?.clone(),
                    )),
                }
            }

            _ => Err(ParserError::UnexpectedToken(tokens.next().unwrap().clone())),
        }
    }
}

#[derive(Debug)]
pub enum Unary {
    Bang(Box<Unary>),
    Negated(Box<Unary>),
    Primary(Box<Primary>),
}
impl AstNode for Unary {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        let unary = match tokens.peek_t() {
            Some(TokenType::Bang) => {
                tokens.next();
                Unary::Bang(Box::new(Unary::parse(tokens)?))
            }
            Some(TokenType::Minus) => {
                tokens.next();
                Unary::Negated(Box::new(Unary::parse(tokens)?))
            }
            _ => Unary::Primary(Box::new(Primary::parse(tokens)?)),
        };
        Ok(unary)
    }
}

#[derive(Debug)]
pub enum Multiplication {
    Mul(Box<Unary>, Option<Box<Multiplication>>),
    Div(Box<Unary>, Option<Box<Multiplication>>),
}
impl AstNode for Multiplication {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        let mut mul = Multiplication::Mul(Box::new(Unary::parse(tokens)?), None);

        while tokens
            .peek_t()
            .map(|t| match t {
                TokenType::Slash | TokenType::Star => true,
                _ => false,
            })
            .unwrap_or(false)
        {
            mul = match tokens.next().unwrap().t {
                TokenType::Star => {
                    Multiplication::Mul(Box::new(Unary::parse(tokens)?), Some(Box::new(mul)))
                }
                TokenType::Slash => {
                    Multiplication::Div(Box::new(Unary::parse(tokens)?), Some(Box::new(mul)))
                }
                _ => return Err(ParserError::LogicError),
            }
        }
        Ok(mul)
    }
}

#[derive(Debug)]
pub enum Addition {
    Add(Box<Multiplication>, Option<Box<Addition>>),
    Sub(Box<Multiplication>, Option<Box<Addition>>),
}
impl AstNode for Addition {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        let mut add = Addition::Add(Box::new(Multiplication::parse(tokens)?), None);

        while tokens
            .peek_t()
            .map(|t| match t {
                TokenType::Minus | TokenType::Plus => true,
                _ => false,
            })
            .unwrap_or(false)
        {
            add = match tokens.next().unwrap().t {
                TokenType::Plus => Addition::Add(
                    Box::new(Multiplication::parse(tokens)?),
                    Some(Box::new(add)),
                ),
                TokenType::Minus => Addition::Sub(
                    Box::new(Multiplication::parse(tokens)?),
                    Some(Box::new(add)),
                ),
                _ => return Err(ParserError::LogicError),
            }
        }
        Ok(add)
    }
}

#[derive(Debug)]
pub enum Comparison {
    G(Box<Addition>, Option<Box<Comparison>>),
    GE(Box<Addition>, Option<Box<Comparison>>),
    L(Box<Addition>, Option<Box<Comparison>>),
    LE(Box<Addition>, Option<Box<Comparison>>),
}
impl AstNode for Comparison {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        let mut com = Comparison::G(Box::new(Addition::parse(tokens)?), None);

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
                TokenType::Greater => {
                    Comparison::G(Box::new(Addition::parse(tokens)?), Some(Box::new(com)))
                }
                TokenType::GreaterEqual => {
                    Comparison::GE(Box::new(Addition::parse(tokens)?), Some(Box::new(com)))
                }
                TokenType::Lesser => {
                    Comparison::L(Box::new(Addition::parse(tokens)?), Some(Box::new(com)))
                }
                TokenType::LesserEqual => {
                    Comparison::LE(Box::new(Addition::parse(tokens)?), Some(Box::new(com)))
                }
                _ => return Err(ParserError::LogicError),
            }
        }
        Ok(com)
    }
}

#[derive(Debug)]
pub enum Equality {
    Eq(Box<Comparison>, Option<Box<Equality>>),
    Not(Box<Comparison>, Option<Box<Equality>>),
}
impl AstNode for Equality {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        let mut eq = Equality::Eq(Box::new(Comparison::parse(tokens)?), None);

        while tokens
            .peek_t()
            .map(|t| match t {
                TokenType::BangEqual | TokenType::EqualEqual => true,
                _ => false,
            })
            .unwrap_or(false)
        {
            eq = match tokens.next().unwrap().t {
                TokenType::EqualEqual => {
                    Equality::Eq(Box::new(Comparison::parse(tokens)?), Some(Box::new(eq)))
                }
                TokenType::BangEqual => {
                    Equality::Not(Box::new(Comparison::parse(tokens)?), Some(Box::new(eq)))
                }
                _ => return Err(ParserError::LogicError),
            }
        }
        Ok(eq)
    }
}

#[derive(Debug)]
pub struct Expression(Box<Equality>);
impl AstNode for Expression {
    fn parse(tokens: &mut dyn TokenIterator) -> ParserResult<Self> {
        Ok(Expression(Box::new(Equality::parse(tokens)?)))
    }
}
