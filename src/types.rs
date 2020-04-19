use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AstType {
    Float,
    Bool,
    Nil,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeError {
    Mismatch(AstType, AstType),
}

impl Ast {
    pub fn annotate_type(&mut self) -> Result<AstType, TypeError> {
        let t = match self {
            Ast::Program(a) => {
                a.annotate_type()?;
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
            Ast::Equal(l, r, t) => {
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
