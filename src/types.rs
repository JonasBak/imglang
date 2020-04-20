use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AstType {
    Float,
    Bool,
    Nil,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeError {
    NotAllowed(AstType),
    Mismatch(AstType, AstType),
}

impl Ast {
    pub fn annotate_type(&mut self) -> Result<AstType, TypeError> {
        let t = match self {
            Ast::Program(ps) => {
                for p in ps.iter_mut() {
                    p.annotate_type()?;
                }
                AstType::Nil
            }
            Ast::Print(expr, t) => {
                let expr_t = match expr.annotate_type()? {
                    t @ AstType::Bool | t @ AstType::Float => t,
                    t @ _ => return Err(TypeError::NotAllowed(t)),
                };
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::ExprStatement(expr, t) => {
                let expr_t = expr.annotate_type()?;
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Float(_) => AstType::Float,
            Ast::Bool(_) => AstType::Bool,
            Ast::Nil => AstType::Nil,
            Ast::Negate(a) => a.annotate_type()?,
            Ast::Not(a) => {
                a.annotate_type()?;
                AstType::Bool
            }
            Ast::Multiply(l, r, t)
            | Ast::Divide(l, r, t)
            | Ast::Add(l, r, t)
            | Ast::Sub(l, r, t) => {
                let t_l = l.annotate_type()?;
                let t_r = r.annotate_type()?;
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
                let t_l = l.annotate_type()?;
                let t_r = r.annotate_type()?;
                if t_l != t_r {
                    return Err(TypeError::Mismatch(t_l, t_r));
                }
                t.replace(t_r);
                AstType::Bool
            }
        };
        Ok(t)
    }
}
