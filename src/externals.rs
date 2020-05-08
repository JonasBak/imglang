use super::*;
use std::collections::HashMap;

pub type ExternalAdr = u16;

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalArg {
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}

pub struct ExternalFunction {
    pub args_t: Vec<AstType>,
    pub ret_t: AstType,
    pub dispatch: fn(args: Vec<ExternalArg>) -> ExternalArg,
}

pub struct Externals {
    functions: Vec<ExternalFunction>,
    function_map: HashMap<String, ExternalAdr>,
}

impl Externals {
    pub fn new() -> Externals {
        Externals {
            functions: Vec::new(),
            function_map: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, name: String, func: ExternalFunction) {
        self.function_map
            .insert(name, self.functions.len() as ExternalAdr);
        self.functions.push(func);
    }

    pub fn lookup_function(&self, name: &String) -> Option<ExternalAdr> {
        self.function_map.get(name).map(|a| *a)
    }

    pub fn lookup_type(&self, name: &String) -> Option<AstType> {
        let i = match self.function_map.get(name) {
            Some(i) => *i,
            None => {
                return None;
            }
        };
        let func = &self.functions[i as usize];
        Some(AstType::ExternalFunction(
            func.args_t.clone(),
            Box::new(func.ret_t.clone()),
        ))
    }

    pub fn dispatch(
        &self,
        adr: ExternalAdr,
        stack: &mut Stack,
        mut offset: StackAdr,
    ) -> ExternalArg {
        let func = self.functions.get(adr as usize).unwrap();
        let mut args = Vec::new();
        for arg_t in func.args_t.iter() {
            args.push(match arg_t {
                AstType::Float => {
                    let f = stack.get(offset).into();
                    ExternalArg::Float(f)
                }
                _ => todo!(),
            });
            offset += 1;
        }
        (func.dispatch)(args)
    }
}
