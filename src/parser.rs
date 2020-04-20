use super::*;

pub enum Ast {
    Program(Vec<Ast>),
    Print(Box<Ast>, Option<AstType>),

    ExprStatement(Box<Ast>, Option<AstType>),

    Float(f64),
    Bool(bool),
    Nil,

    Negate(Box<Ast>),
    Not(Box<Ast>),

    Multiply(Box<Ast>, Box<Ast>, Option<AstType>),
    Divide(Box<Ast>, Box<Ast>, Option<AstType>),
    Add(Box<Ast>, Box<Ast>, Option<AstType>),
    Sub(Box<Ast>, Box<Ast>, Option<AstType>),

    Equal(Box<Ast>, Box<Ast>, Option<AstType>),
    NotEqual(Box<Ast>, Box<Ast>, Option<AstType>),
    Greater(Box<Ast>, Box<Ast>, Option<AstType>),
    GreaterEqual(Box<Ast>, Box<Ast>, Option<AstType>),
    Lesser(Box<Ast>, Box<Ast>, Option<AstType>),
    LesserEqual(Box<Ast>, Box<Ast>, Option<AstType>),
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
        TokenType::Float(_) => (Some(literal), None, PREC_NONE),
        TokenType::Star => (None, Some(binary), PREC_FACTOR),
        TokenType::Slash => (None, Some(binary), PREC_FACTOR),
        TokenType::Plus => (None, Some(binary), PREC_TERM),
        TokenType::Minus => (Some(unary), Some(binary), PREC_TERM),
        TokenType::True => (Some(literal), None, PREC_NONE),
        TokenType::False => (Some(literal), None, PREC_NONE),
        TokenType::Nil => (Some(literal), None, PREC_NONE),
        TokenType::Bang => (Some(unary), None, PREC_NONE),
        TokenType::EqualEqual => (None, Some(binary), PREC_EQUALITY),
        TokenType::BangEqual => (None, Some(binary), PREC_EQUALITY),
        TokenType::Greater => (None, Some(binary), PREC_COMPARISON),
        TokenType::GreaterEqual => (None, Some(binary), PREC_COMPARISON),
        TokenType::Lesser => (None, Some(binary), PREC_COMPARISON),
        TokenType::LesserEqual => (None, Some(binary), PREC_COMPARISON),
        _ => (None, None, PREC_NONE),
    }
}

pub fn parse(lexer: &mut Lexer) -> Ast {
    let mut parsed = vec![];
    while lexer.current_t() != TokenType::Eof {
        parsed.push(declaration(lexer));
    }
    Ast::Program(parsed)
}

fn parse_precedence(lexer: &mut Lexer, prec: u32) -> Ast {
    lexer.next().unwrap();

    let prefix_rule = get_rule(&lexer.prev_t().unwrap()).0.unwrap();

    let mut lhs = prefix_rule(lexer);

    while prec <= get_rule(&lexer.current_t()).2 {
        lexer.next();
        let infix_rule = get_rule(&lexer.prev_t().unwrap()).1.unwrap();
        lhs = infix_rule(lexer, lhs);
    }

    lhs
}

fn literal(lexer: &mut Lexer) -> Ast {
    match lexer.prev_t().unwrap() {
        TokenType::Float(f) => Ast::Float(f),
        TokenType::True => Ast::Bool(true),
        TokenType::False => Ast::Bool(false),
        TokenType::Nil => Ast::Nil,
        _ => todo!(
            "parsing literal of type {:?} not implemented",
            lexer.prev_t()
        ),
    }
}

fn expression(lexer: &mut Lexer) -> Ast {
    parse_precedence(lexer, PREC_ASSIGNMENT)
}

fn unary(lexer: &mut Lexer) -> Ast {
    let t = lexer.prev_t().unwrap();
    let expr = parse_precedence(lexer, PREC_UNARY);
    match t {
        TokenType::Minus => Ast::Negate(Box::new(expr)),
        TokenType::Bang => Ast::Not(Box::new(expr)),
        _ => todo!(),
    }
}

fn binary(lexer: &mut Lexer, lhs: Ast) -> Ast {
    let t = lexer.prev_t().unwrap();
    let rule = get_rule(&t);
    let rhs = parse_precedence(lexer, rule.2 + 1);
    match t {
        TokenType::Star => Ast::Multiply(Box::new(lhs), Box::new(rhs), None),
        TokenType::Slash => Ast::Divide(Box::new(lhs), Box::new(rhs), None),
        TokenType::Plus => Ast::Add(Box::new(lhs), Box::new(rhs), None),
        TokenType::Minus => Ast::Sub(Box::new(lhs), Box::new(rhs), None),
        TokenType::EqualEqual => Ast::Equal(Box::new(lhs), Box::new(rhs), None),
        TokenType::BangEqual => Ast::NotEqual(Box::new(lhs), Box::new(rhs), None),
        TokenType::Greater => Ast::Greater(Box::new(lhs), Box::new(rhs), None),
        TokenType::GreaterEqual => Ast::GreaterEqual(Box::new(lhs), Box::new(rhs), None),
        TokenType::Lesser => Ast::Lesser(Box::new(lhs), Box::new(rhs), None),
        TokenType::LesserEqual => Ast::LesserEqual(Box::new(lhs), Box::new(rhs), None),
        _ => todo!(),
    }
}

fn grouping(lexer: &mut Lexer) -> Ast {
    let expr = expression(lexer);
    consume(lexer, |t| t == &TokenType::RightPar);
    expr
}

fn declaration(lexer: &mut Lexer) -> Ast {
    statement(lexer)
}

fn statement(lexer: &mut Lexer) -> Ast {
    match lexer.current_t() {
        TokenType::Print => {
            lexer.next();
            print_statement(lexer)
        }
        _ => expression_statement(lexer),
    }
}

fn print_statement(lexer: &mut Lexer) -> Ast {
    let expr = expression(lexer);
    consume(lexer, |t| t == &TokenType::Semicolon);
    Ast::Print(Box::new(expr), None)
}

fn expression_statement(lexer: &mut Lexer) -> Ast {
    let expr = expression(lexer);
    consume(lexer, |t| t == &TokenType::Semicolon);
    Ast::ExprStatement(Box::new(expr), None)
}
