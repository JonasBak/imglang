use super::*;

#[derive(Debug, Clone)]
pub enum Ast {
    Program(Vec<Ast>),
    Block {
        cont: Vec<Ast>,
        pos: usize,
    },
    Print {
        expr: Box<Ast>,
        t: Option<AstType>,
        pos: usize,
    },
    Return {
        expr: Option<Box<Ast>>,
        t: Option<AstType>,
        pos: usize,
    },

    Declaration {
        name: String,
        expr: Box<Ast>,
        t: Option<AstType>,
        pos: usize,
    },
    FuncDeclaration {
        name: String,
        func: Box<Ast>,
        args_t: Vec<AstType>,
        ret_t: AstType,
        pos: usize,
    },
    EnumDeclaration {
        name: String,
        variants: Vec<(String, AstType)>,
        pos: usize,
    },

    Variable {
        name: String,
        t: Option<AstType>,
        pos: usize,
    },
    Assign {
        name: String,
        expr: Box<Ast>,
        t: Option<AstType>,
        move_to_heap: Option<bool>,
        pos: usize,
    },

    If {
        condition: Box<Ast>,
        body: Box<Ast>,
        else_body: Option<Box<Ast>>,
        pos: usize,
    },
    While {
        condition: Box<Ast>,
        body: Box<Ast>,
        pos: usize,
    },

    ExprStatement {
        expr: Box<Ast>,
        t: Option<AstType>,
        pos: usize,
    },

    Function {
        body: Box<Ast>,
        args: Vec<(String, AstType)>,
        captured: Vec<(String, Option<AstType>)>,
        ret_t: AstType,
        pos: usize,
    },
    Call {
        ident: Box<Ast>,
        args: Vec<Ast>,
        args_width: Option<u8>,
        call_t: Option<CallType>,
        pos: usize,
    },

    Float(f64, usize),
    Bool(bool, usize),

    String(String, usize),

    Negate(Box<Ast>, usize),
    Not(Box<Ast>, usize),

    Multiply(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    Divide(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    Add(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    Sub(Box<Ast>, Box<Ast>, Option<AstType>, usize),

    Equal(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    NotEqual(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    Greater(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    GreaterEqual(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    Lesser(Box<Ast>, Box<Ast>, Option<AstType>, usize),
    LesserEqual(Box<Ast>, Box<Ast>, Option<AstType>, usize),

    And(Box<Ast>, Box<Ast>, usize),
    Or(Box<Ast>, Box<Ast>, usize),
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

fn consume(lexer: &mut Lexer, p: fn(&TokenType) -> bool, msg: &'static str) -> ParserResult<()> {
    if !p(&lexer.current_t()) {
        return Err(ParserError::Unexpected(lexer.current(), msg));
    }
    lexer.next();
    Ok(())
}

fn parse_type(lexer: &mut Lexer) -> ParserResult<Option<AstType>> {
    let t = match lexer.current_t() {
        TokenType::TypeFloat => {
            lexer.next();
            AstType::Float
        }
        TokenType::TypeBool => {
            lexer.next();
            AstType::Bool
        }
        TokenType::TypeString => {
            lexer.next();
            AstType::String
        }
        TokenType::TypeNil => {
            lexer.next();
            AstType::Nil
        }
        TokenType::Lesser => {
            lexer.next();
            let mut args = vec![];
            while match lexer.current_t() {
                TokenType::Semicolon | TokenType::Greater => false,
                _ => true,
            } {
                args.push(parse_type(lexer)?.unwrap());
                if lexer.current_t() == TokenType::Comma {
                    lexer.next();
                }
            }

            let ret_t = if lexer.current_t() == TokenType::Semicolon {
                lexer.next();
                parse_type(lexer)?.unwrap()
            } else {
                AstType::Nil
            };

            consume(
                lexer,
                |t| t == &TokenType::Greater,
                "expected '>' after argument types",
            )?;
            if lexer.current_t() == TokenType::Star {
                lexer.next();
                AstType::Closure(args, Box::new(ret_t))
            } else {
                AstType::Function(args, Box::new(ret_t))
            }
        }
        _ => return Ok(None),
    };
    return Ok(Some(t));
}

fn get_rule(t: &TokenType) -> Rule {
    match t {
        TokenType::LeftPar => (Some(grouping), Some(call), PREC_CALL),
        TokenType::Float(_) => (Some(literal), None, PREC_NONE),
        TokenType::Star => (None, Some(binary), PREC_FACTOR),
        TokenType::Slash => (None, Some(binary), PREC_FACTOR),
        TokenType::Plus => (None, Some(binary), PREC_TERM),
        TokenType::Minus => (Some(unary), Some(binary), PREC_TERM),
        TokenType::True => (Some(literal), None, PREC_NONE),
        TokenType::False => (Some(literal), None, PREC_NONE),
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
        TokenType::Fun => (Some(function), None, PREC_NONE),
        TokenType::String(_) => (Some(literal), None, PREC_NONE),
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
        ParserError::Unexpected(lexer.prev().unwrap(), "unexpected token in prefix position")
    })?;

    let mut lhs = prefix_rule(lexer)?;

    while prec <= get_rule(&lexer.current_t()).2 {
        lexer.next();
        let infix_rule = get_rule(&lexer.prev_t().unwrap()).1.ok_or_else(|| {
            ParserError::Unexpected(lexer.prev().unwrap(), "unexpected token in infix position")
        })?;
        lhs = infix_rule(lexer, lhs)?;
    }

    Ok(lhs)
}

fn literal(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let ast = match lexer.prev_t().unwrap() {
        TokenType::Float(f) => Ast::Float(f, pos),
        TokenType::True => Ast::Bool(true, pos),
        TokenType::False => Ast::Bool(false, pos),
        TokenType::String(s) => Ast::String(s, pos),
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
    let pos = lexer.prev().unwrap().start;
    let t = lexer.prev_t().unwrap();
    let expr = parse_precedence(lexer, PREC_UNARY)?;
    let ast = match t {
        TokenType::Minus => Ast::Negate(Box::new(expr), pos),
        TokenType::Bang => Ast::Not(Box::new(expr), pos),
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
    let pos = lexer.prev().unwrap().start;
    let t = lexer.prev_t().unwrap();
    let rule = get_rule(&t);
    let rhs = parse_precedence(lexer, rule.2 + 1)?;
    let ast = match t {
        TokenType::Star => Ast::Multiply(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::Slash => Ast::Divide(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::Plus => Ast::Add(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::Minus => Ast::Sub(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::EqualEqual => Ast::Equal(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::BangEqual => Ast::NotEqual(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::Greater => Ast::Greater(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::GreaterEqual => Ast::GreaterEqual(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::Lesser => Ast::Lesser(Box::new(lhs), Box::new(rhs), None, pos),
        TokenType::LesserEqual => Ast::LesserEqual(Box::new(lhs), Box::new(rhs), None, pos),
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
        TokenType::Fun => {
            lexer.next();
            func_declaration(lexer)
        }
        TokenType::Enum => {
            lexer.next();
            enum_declaration(lexer)
        }
        _ => statement(lexer),
    }
}

fn var_declaration(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
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

    Ok(Ast::Declaration {
        name,
        expr: Box::new(expr),
        t: None,
        pos,
    })
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
    let pos = lexer.prev().unwrap().start;
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
            Ok(Ast::Assign {
                name,
                expr: Box::new(expr),
                t: None,
                move_to_heap: None,
                pos,
            })
        }
        _ => Ok(Ast::Variable { name, t: None, pos }),
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
        TokenType::While => {
            lexer.next();
            while_statement(lexer)
        }
        TokenType::Return => {
            lexer.next();
            return_statement(lexer)
        }
        _ => expression_statement(lexer),
    }
}

fn print_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let expr = expression(lexer)?;
    consume(
        lexer,
        |t| t == &TokenType::Semicolon,
        "expected ';' after print statement",
    )?;
    Ok(Ast::Print {
        expr: Box::new(expr),
        t: None,
        pos,
    })
}

fn if_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
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
    Ok(Ast::If {
        condition: Box::new(expr),
        body: Box::new(stmt),
        else_body: else_stmt,
        pos,
    })
}

fn block(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
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
        Ok(Ast::Block { cont: parsed, pos })
    }
}

fn expression_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let expr = expression(lexer)?;
    consume(lexer, |t| t == &TokenType::Semicolon, "expected ';'")?;
    Ok(Ast::ExprStatement {
        expr: Box::new(expr),
        t: None,
        pos,
    })
}

fn logic_and(lexer: &mut Lexer, lhs: Ast) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let rhs = parse_precedence(lexer, PREC_AND)?;
    Ok(Ast::And(Box::new(lhs), Box::new(rhs), pos))
}

fn logic_or(lexer: &mut Lexer, lhs: Ast) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let rhs = parse_precedence(lexer, PREC_OR)?;
    Ok(Ast::Or(Box::new(lhs), Box::new(rhs), pos))
}

fn while_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    consume(
        lexer,
        |t| t == &TokenType::LeftPar,
        "expected '(' after while",
    )?;
    let expr = expression(lexer)?;
    consume(
        lexer,
        |t| t == &TokenType::RightPar,
        "expected ')' after condition",
    )?;
    let stmt = statement(lexer)?;

    Ok(Ast::While {
        condition: Box::new(expr),
        body: Box::new(stmt),
        pos,
    })
}

fn function(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let mut captured = Vec::new();
    if lexer.current_t() == TokenType::LeftSquare {
        lexer.next();
        while lexer.current_t() != TokenType::RightSquare {
            let var = match lexer.current_t() {
                TokenType::Identifier(var) => var,
                _ => todo!(),
            };
            captured.push((var, None));

            lexer.next();

            if lexer.current_t() != TokenType::Comma {
                break;
            }
            lexer.next();
        }
        consume(
            lexer,
            |t| t == &TokenType::RightSquare,
            "expected ']' after captured variables",
        )?;
    }
    consume(
        lexer,
        |t| t == &TokenType::LeftPar,
        "expected '(' before arguments",
    )?;
    let mut args = vec![];
    while lexer.current_t() != TokenType::RightPar {
        let arg = match lexer.current_t() {
            TokenType::Identifier(arg) => arg,
            _ => todo!(),
        };
        lexer.next();

        let arg_t = parse_type(lexer)?.ok_or_else(|| {
            ParserError::Unexpected(lexer.current(), "expected type after argument")
        })?;

        args.push((arg, arg_t));

        if lexer.current_t() != TokenType::Comma {
            break;
        }
        lexer.next();
    }
    consume(
        lexer,
        |t| t == &TokenType::RightPar,
        "expected ')' after arguments",
    )?;
    let ret_t = parse_type(lexer)?.unwrap_or(AstType::Nil);
    let body = if lexer.current_t() == TokenType::LeftBrace {
        statement(lexer)?
    } else {
        let ret_expr = expression(lexer)?;
        Ast::Block {
            cont: vec![Ast::Return {
                expr: Some(Box::new(ret_expr)),
                t: None,
                pos,
            }],
            pos,
        }
    };
    Ok(Ast::Function {
        body: Box::new(body),
        args,
        captured,
        ret_t,
        pos,
    })
}

fn call(lexer: &mut Lexer, ident: Ast) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let mut args = vec![];
    while lexer.current_t() != TokenType::RightPar {
        args.push(expression(lexer)?);
        if lexer.current_t() == TokenType::Comma {
            lexer.next();
        }
    }
    consume(
        lexer,
        |t| t == &TokenType::RightPar,
        "expected ')' after arguments",
    )?;
    Ok(Ast::Call {
        ident: Box::new(ident),
        args,
        args_width: None,
        call_t: None,
        pos,
    })
}

fn func_declaration(lexer: &mut Lexer) -> ParserResult<Ast> {
    let name = parse_variable(lexer)?;

    let function = match function(lexer)? {
        Ast::Function {
            body,
            args,
            captured,
            ret_t,
            pos,
        } => {
            if captured.len() > 0 {
                return Err(ParserError::Unexpected(
                    lexer.prev().unwrap(),
                    "function declarations can't be closures",
                ));
            }

            Ast::FuncDeclaration {
                name,
                func: Box::new(Ast::Function {
                    body,
                    args: args.clone(),
                    captured,
                    ret_t: ret_t.clone(),
                    pos,
                }),
                args_t: args.into_iter().map(|a| a.1).collect(),
                ret_t,
                pos,
            }
        }
        _ => panic!(),
    };

    Ok(function)
}

fn enum_declaration(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let name = match lexer.current_t() {
        TokenType::Identifier(name) => name,
        _ => {
            return Err(ParserError::Unexpected(
                lexer.current(),
                "unexpected token when parsing enum, expected identifier",
            ))
        }
    };
    lexer.next();
    consume(
        lexer,
        |t| t == &TokenType::Equal,
        "expected '=' after enum name",
    )?;
    let mut variants = Vec::new();
    while lexer.current_t() != TokenType::Semicolon {
        let variant = match lexer.current_t() {
            TokenType::Identifier(variant) => variant,
            _ => {
                return Err(ParserError::Unexpected(
                    lexer.current(),
                    "unexpected token when parsing enum variants, expected identifier",
                ))
            }
        };
        lexer.next();
        let t = parse_type(lexer)?.unwrap_or(AstType::Nil);
        variants.push((variant, t));
        if lexer.current_t() == TokenType::Bar {
            lexer.next();
        }
    }
    lexer.next();
    Ok(Ast::EnumDeclaration {
        name,
        variants,
        pos,
    })
}

fn return_statement(lexer: &mut Lexer) -> ParserResult<Ast> {
    let pos = lexer.prev().unwrap().start;
    let expr = if lexer.current_t() != TokenType::Semicolon {
        Some(Box::new(expression(lexer)?))
    } else {
        None
    };
    consume(
        lexer,
        |t| t == &TokenType::Semicolon,
        "expected ';' after return statement",
    )?;
    Ok(Ast::Return { expr, t: None, pos })
}
