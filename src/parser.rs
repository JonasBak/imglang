use super::*;

#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>),
    Block(Vec<Ast>),
    Print(Box<Ast>, Option<AstType>),

    Declaration(String, Box<Ast>, Option<AstType>),
    Variable(String, Option<AstType>),
    Assign(String, Box<Ast>, Option<AstType>),

    If(Box<Ast>, Box<Ast>, Option<Box<Ast>>),

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

    And(Box<Ast>, Box<Ast>),
    Or(Box<Ast>, Box<Ast>),
}

type ParserResult<T> = Result<T, ParserError>;
#[derive(Debug)]
pub enum ParserError {
    Unexpected(Token, &'static str),
    BlockErrors(Vec<ParserError>),
}

type Rule = (
    Option<fn(&mut Lexer) -> ParserResult<Ast>>,
    Option<fn(&mut Lexer, Ast) -> ParserResult<Ast>>,
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

fn consume(lexer: &mut Lexer, p: fn(&TokenType) -> bool, msg: &'static str) -> ParserResult<()> {
    if !p(&lexer.current_t()) {
        return Err(ParserError::Unexpected(lexer.current(), msg));
    }
    lexer.next();
    Ok(())
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
        TokenType::Identifier(_) => (Some(variable), None, PREC_NONE),
        TokenType::And => (None, Some(logic_and), PREC_AND),
        TokenType::Or => (None, Some(logic_or), PREC_OR),
        _ => (None, None, PREC_NONE),
    }
}

fn synchronize(lexer: &mut Lexer) {
    while lexer.current_t() != TokenType::Eof {
        if let Some(TokenType::Semicolon) = lexer.prev_t() {
            return;
        }
        match lexer.current_t() {
            TokenType::Var | TokenType::Print => return,
            _ => {}
        }
        lexer.next();
    }
}

pub fn parse(lexer: &mut Lexer) -> ParserResult<Ast> {
    let mut parsed = vec![];
    let mut errors = vec![];
    while lexer.current_t() != TokenType::Eof {
        match declaration(lexer) {
            Ok(decl) => parsed.push(decl),
            Err(error) => {
                errors.push(error);
                synchronize(lexer);
            }
        }
    }
    if errors.len() > 0 {
        Err(ParserError::BlockErrors(errors))
    } else {
        Ok(Ast::Program(parsed))
    }
}

fn parse_precedence(lexer: &mut Lexer, prec: u32) -> ParserResult<Ast> {
    lexer.next().unwrap();

    let prefix_rule = get_rule(&lexer.prev_t().unwrap()).0.ok_or_else(|| {
        ParserError::Unexpected(
            lexer.prev().unwrap(),
            "unexpected token in prefix prosition",
        )
    })?;

    let mut lhs = prefix_rule(lexer)?;

    while prec <= get_rule(&lexer.current_t()).2 {
        lexer.next();
        let infix_rule = get_rule(&lexer.prev_t().unwrap()).1.ok_or_else(|| {
            ParserError::Unexpected(lexer.prev().unwrap(), "unexpected token in infix prosition")
        })?;
        lhs = infix_rule(lexer, lhs)?;
    }

    Ok(lhs)
}

fn literal(lexer: &mut Lexer) -> ParserResult<Ast> {
    let ast = match lexer.prev_t().unwrap() {
        TokenType::Float(f) => Ast::Float(f),
        TokenType::True => Ast::Bool(true),
        TokenType::False => Ast::Bool(false),
        TokenType::Nil => Ast::Nil,
        _ => {
            return Err(ParserError::Unexpected(
                lexer.prev().unwrap(),
                "unexpected token when parsing literal",
            ))
        }
    };
    Ok(ast)
}

fn expression(lexer: &mut Lexer) -> ParserResult<Ast> {
    parse_precedence(lexer, PREC_ASSIGNMENT)
}

fn unary(lexer: &mut Lexer) -> ParserResult<Ast> {
    let t = lexer.prev_t().unwrap();
    let expr = parse_precedence(lexer, PREC_UNARY)?;
    let ast = match t {
        TokenType::Minus => Ast::Negate(Box::new(expr)),
        TokenType::Bang => Ast::Not(Box::new(expr)),
        _ => {
            return Err(ParserError::Unexpected(
                lexer.prev().unwrap(),
                "unexpected token when parsing unary expression",
            ))
        }
    };
    Ok(ast)
}

fn binary(lexer: &mut Lexer, lhs: Ast) -> ParserResult<Ast> {
    let t = lexer.prev_t().unwrap();
    let rule = get_rule(&t);
    let rhs = parse_precedence(lexer, rule.2 + 1)?;
    let ast = match t {
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
        _ => {
            return Err(ParserError::Unexpected(
                lexer.prev().unwrap(),
                "unexpected token when parsing binary expression",
            ))
        }
    };
    Ok(ast)
}

fn grouping(lexer: &mut Lexer) -> ParserResult<Ast> {
    let expr = expression(lexer)?;
    consume(
        lexer,
        |t| t == &TokenType::RightPar,
        "expected ')' after grouping",
    )?;
    Ok(expr)
}

fn declaration(lexer: &mut Lexer) -> ParserResult<Ast> {
    match lexer.current_t() {
        TokenType::Var => {
            lexer.next();
            var_declaration(lexer)
        }
        _ => statement(lexer),
    }
}

fn var_declaration(lexer: &mut Lexer) -> ParserResult<Ast> {
    let name = parse_variable(lexer)?;

    consume(
        lexer,
        |t| t == &TokenType::Equal,
        "expected ' = [expression];'",
    )?;
    let expr = expression(lexer)?;

    consume(
        lexer,
        |t| t == &TokenType::Semicolon,
        "expected ';' after declaration",
    )?;

    Ok(Ast::Declaration(name, Box::new(expr), None))
}

fn parse_variable(lexer: &mut Lexer) -> ParserResult<String> {
    match lexer.current_t() {
        TokenType::Identifier(s) => {
            lexer.next();
            Ok(s)
        }
        _ => {
            return Err(ParserError::Unexpected(
                lexer.current(),
                "unexpected token when parsing variable, expected identifier",
            ))
        }
    }
}

fn variable(lexer: &mut Lexer) -> ParserResult<Ast> {
    named_variable(lexer)
}

fn named_variable(lexer: &mut Lexer) -> ParserResult<Ast> {
    let name = match lexer.prev_t().unwrap() {
        TokenType::Identifier(name) => name,
        _ => {
            return Err(ParserError::Unexpected(
                lexer.prev().unwrap(),
                "unexpected token when parsing variable, expected identifier",
            ))
        }
    };
    match lexer.current_t() {
        TokenType::Equal => {
            lexer.next();
            let expr = expression(lexer)?;
            Ok(Ast::Assign(name, Box::new(expr), None))
        }
        _ => Ok(Ast::Variable(name, None)),
    }
}

fn statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    match lexer.current_t() {
        TokenType::Print => {
            lexer.next();
            print_statement(lexer)
        }
        TokenType::LeftBrace => {
            lexer.next();
            block(lexer)
        }
        TokenType::If => {
            lexer.next();
            if_statement(lexer)
        }
        _ => expression_statement(lexer),
    }
}

fn print_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let expr = expression(lexer)?;
    consume(
        lexer,
        |t| t == &TokenType::Semicolon,
        "expected ';' after print statement",
    )?;
    Ok(Ast::Print(Box::new(expr), None))
}

fn if_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    consume(lexer, |t| t == &TokenType::LeftPar, "expected '(' after if")?;
    let expr = expression(lexer)?;
    consume(
        lexer,
        |t| t == &TokenType::RightPar,
        "expected ')' after condition",
    )?;
    let stmt = statement(lexer)?;
    let else_stmt = match lexer.current_t() {
        TokenType::Else => {
            lexer.next();
            Some(Box::new(statement(lexer)?))
        }
        _ => None,
    };
    Ok(Ast::If(Box::new(expr), Box::new(stmt), else_stmt))
}

fn block(lexer: &mut Lexer) -> ParserResult<Ast> {
    let mut parsed = vec![];
    let mut errors = vec![];
    while match lexer.current_t() {
        TokenType::RightBrace | TokenType::Eof => false,
        _ => true,
    } {
        match declaration(lexer) {
            Ok(decl) => parsed.push(decl),
            Err(error) => {
                errors.push(error);
                synchronize(lexer);
            }
        }
    }
    if let Err(error) = consume(
        lexer,
        |t| t == &TokenType::RightBrace,
        "expected '}' after block",
    ) {
        errors.push(error);
        synchronize(lexer);
    }
    if errors.len() > 0 {
        Err(ParserError::BlockErrors(errors))
    } else {
        Ok(Ast::Block(parsed))
    }
}

fn expression_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let expr = expression(lexer)?;
    consume(lexer, |t| t == &TokenType::Semicolon, "expected ';'")?;
    Ok(Ast::ExprStatement(Box::new(expr), None))
}

fn logic_and(lexer: &mut Lexer, lhs: Ast) -> ParserResult<Ast> {
    let rhs = parse_precedence(lexer, PREC_AND)?;
    Ok(Ast::And(Box::new(lhs), Box::new(rhs)))
}

fn logic_or(lexer: &mut Lexer, lhs: Ast) -> ParserResult<Ast> {
    let rhs = parse_precedence(lexer, PREC_OR)?;
    Ok(Ast::Or(Box::new(lhs), Box::new(rhs)))
}
