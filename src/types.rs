use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AstType {
    Float,
    Bool,
    Nil,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    NotAllowed(AstType),
    Mismatch(AstType, AstType),
    NotDefined(String),
}

struct Variable {
    name: String,
    depth: u16,
    t: AstType,
}
pub struct TypeChecker {
    variables: Vec<Variable>,
    current_scope_depth: u16,
}

impl TypeChecker {
    pub fn annotate_types(ast: &mut Ast) -> Result<(), TypeError> {
        let mut type_checker = TypeChecker {
            variables: vec![],
            current_scope_depth: 0,
        };
        type_checker.annotate_type(ast)?;
        Ok(())
    }
    fn declare_variable(&mut self, name: &String, t: AstType) {
        self.variables.push(Variable {
            name: name.clone(),
            depth: self.current_scope_depth,
            t,
        });
    }
    fn resolve_variable(&mut self, name: &String) -> Option<&Variable> {
        self.variables.iter().rev().find(|v| &v.name == name)
    }
    fn annotate_type(&mut self, ast: &mut Ast) -> Result<AstType, TypeError> {
        let t = match ast {
            Ast::Program(ps) => {
                for p in ps.iter_mut() {
                    self.annotate_type(p)?;
                }
                AstType::Nil
            }
            Ast::Block(ps) => {
                self.current_scope_depth += 1;
                for p in ps.iter_mut() {
                    self.annotate_type(p)?;
                }
                self.current_scope_depth -= 1;
                while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth
                {
                    self.variables.pop();
                }
                AstType::Nil
            }
            Ast::Print(expr, t) => {
                let expr_t = match self.annotate_type(expr)? {
                    t @ AstType::Bool | t @ AstType::Float => t,
                    t @ _ => return Err(TypeError::NotAllowed(t)),
                };
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Declaration(name, expr, t) => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t);
                self.declare_variable(name, expr_t);
                AstType::Nil
            }
            Ast::Variable(name, t) => {
                let v = self
                    .resolve_variable(name)
                    .ok_or(TypeError::NotDefined(name.clone()))?;
                t.replace(v.t);
                v.t
            }
            Ast::Assign(name, expr, t) => {
                let v_t = self
                    .resolve_variable(name)
                    .ok_or(TypeError::NotDefined(name.clone()))?
                    .t;
                let expr_t = self.annotate_type(expr)?;
                if v_t != expr_t {
                    return Err(TypeError::Mismatch(v_t, expr_t));
                }
                t.replace(expr_t);
                v_t
            }
            Ast::If(expr, stmt, else_stmt) => {
                let expr_t = self.annotate_type(expr)?;
                if expr_t != AstType::Bool {
                    return Err(TypeError::NotAllowed(expr_t));
                }
                self.annotate_type(stmt)?;
                if let Some(else_stmt) = else_stmt {
                    self.annotate_type(else_stmt)?;
                }
                AstType::Nil
            }
            Ast::ExprStatement(expr, t) => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Float(_) => AstType::Float,
            Ast::Bool(_) => AstType::Bool,
            Ast::Nil => AstType::Nil,
            Ast::Negate(a) => self.annotate_type(a)?,
            Ast::Not(a) => {
                self.annotate_type(a)?;
                AstType::Bool
            }
            Ast::Multiply(l, r, t)
            | Ast::Divide(l, r, t)
            | Ast::Add(l, r, t)
            | Ast::Sub(l, r, t) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Mismatch(t_l, t_r));
                }
                t.replace(t_r);
                t_r
            }
            Ast::Equal(l, r, t)
            | Ast::NotEqual(l, r, t)
            | Ast::Greater(l, r, t)
            | Ast::GreaterEqual(l, r, t)
            | Ast::Lesser(l, r, t)
            | Ast::LesserEqual(l, r, t) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Mismatch(t_l, t_r));
                }
                t.replace(t_r);
                AstType::Bool
            }
            Ast::And(l, r) | Ast::Or(l, r) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != AstType::Bool {
                    return Err(TypeError::NotAllowed(t_l));
                }
                if t_r != AstType::Bool {
                    return Err(TypeError::NotAllowed(t_r));
                }
                AstType::Bool
            }
        };
        Ok(t)
    }
}
