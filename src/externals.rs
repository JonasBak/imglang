use super::*;
use std::collections::HashMap;

pub type ExternalAdr = u16;

pub struct ExternalFunction {
    pub args_t: Vec<AstType>,
    pub ret_t: AstType,
    pub dispatch: fn(&mut Stack),
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

    pub fn dispatch(&self, adr: ExternalAdr, stack: &mut Stack) {
        let func = self.functions.get(adr as usize).unwrap();
        (func.dispatch)(stack);
    }
}

pub trait AstTypeCaster {
    fn ast_type() -> AstType;
}

macro_rules! external_pop_args {
    ($stack:ident, ($arg:ident, $t:ident), $(($rest_arg:ident, $rest_t:ident)),+) => {
        external_pop_args!($stack, $(($rest_arg, $rest_t)),+);
        let $arg: $t = $stack.pop();
    };
    ($stack:ident, ($arg:ident, $t:ident)) => {
        let $arg: $t = $stack.pop();
    };
}
