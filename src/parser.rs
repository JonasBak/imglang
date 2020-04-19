use super::*;

pub enum Ast {
    Program(Box<Ast>),

    Float(f64),
    Negate(Box<Ast>),

    Multiply(Box<Ast>, Box<Ast>),
    Divide(Box<Ast>, Box<Ast>),
    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),
}

type Rule = (
    Option<fn(&mut Lexer) -> Ast>,
    Option<fn(&mut Lexer, Ast) -> Ast>,
    u32,
);

pub const PREC_NONE: u32 = 0;
pub const PREC_ASSIGNMENT: u32 = 10; // =
pub const PREC_OR: u32 = 20; // or
pub const PREC_AND: u32 = 30; // and
pub const PREC_EQUALITY: u32 = 40; // == !=
pub const PREC_COMPARISON: u32 = 50; // < > <= >=
pub const PREC_TERM: u32 = 60; // + -
pub const PREC_FACTOR: u32 = 70; // * /
pub const PREC_UNARY: u32 = 80; // ! -
pub const PREC_CALL: u32 = 90; // . ()
pub const PREC_PRIMARY: u32 = 100;

fn consume(lexer: &mut Lexer, p: fn(&TokenType) -> bool) {
    if !p(&lexer.current_t()) {
        todo!();
    }
    lexer.next();
}

fn get_rule(t: &TokenType) -> Rule {
    match t {
        TokenType::LeftPar => (Some(grouping), None, PREC_NONE),
        TokenType::Float(_) => (Some(float), None, PREC_NONE),
        TokenType::Star => (None, Some(binary), PREC_FACTOR),
        TokenType::Slash => (None, Some(binary), PREC_FACTOR),
        TokenType::Plus => (None, Some(binary), PREC_TERM),
        TokenType::Minus => (Some(unary), Some(binary), PREC_TERM),
        _ => (None, None, PREC_NONE),
    }
}

pub fn parse(lexer: &mut Lexer) -> Ast {
    Ast::Program(Box::new(parse_precedence(lexer, PREC_NONE)))
}

fn parse_precedence(lexer: &mut Lexer, prec: u32) -> Ast {
    lexer.next().unwrap();

    let prefix_rule = get_rule(&lexer.prev_t()).0.unwrap();

    let mut lhs = prefix_rule(lexer);

    while prec <= get_rule(&lexer.current_t()).2 {
        if lexer.next().is_none() {
            break;
        }
        let infix_rule = get_rule(&lexer.prev_t()).1.unwrap();
        lhs = infix_rule(lexer, lhs);
    }

    lhs
}

fn float(lexer: &mut Lexer) -> Ast {
    match lexer.prev_t() {
        TokenType::Float(f) => Ast::Float(f),
        _ => todo!(),
    }
}

fn expression(lexer: &mut Lexer) -> Ast {
    parse_precedence(lexer, PREC_ASSIGNMENT)
}

fn unary(lexer: &mut Lexer) -> Ast {
    let t = lexer.prev_t();
    let expr = parse_precedence(lexer, PREC_UNARY);
    match t {
        TokenType::Minus => Ast::Negate(Box::new(expr)),
        _ => todo!(),
    }
}

fn binary(lexer: &mut Lexer, lhs: Ast) -> Ast {
    let t = lexer.prev_t();
    let rule = get_rule(&t);
    let rhs = parse_precedence(lexer, rule.2 + 1);
    match t {
        TokenType::Star => Ast::Multiply(Box::new(lhs), Box::new(rhs)),
        TokenType::Slash => Ast::Divide(Box::new(lhs), Box::new(rhs)),
        TokenType::Plus => Ast::Add(Box::new(lhs), Box::new(rhs)),
        TokenType::Minus => Ast::Sub(Box::new(lhs), Box::new(rhs)),
        _ => todo!(),
    }
}

fn grouping(lexer: &mut Lexer) -> Ast {
    let expr = expression(lexer);
    consume(lexer, |t| t == &TokenType::RightPar);
    expr
}
