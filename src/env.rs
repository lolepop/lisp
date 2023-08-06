use std::{cell::RefCell, any::Any};

use rustc_hash::FxHashMap;

use crate::Ast;

type ArgStack = Vec<EnvType>;

// #[derive(Debug)]
// struct ProcInfo<'a> {
//     args: u8,
//     body: &'a Ast,
//     captured: &'a Env, // need to make env have interior mutability
// }

#[derive(Debug, Clone)]
pub enum EnvType {
    Number(f64),
    // Proc(String, ProcInfo<'a>),
    NativeProc(String),
    List(Vec<EnvType>),
}

#[derive(Debug)]
pub struct Env {
    variables: FxHashMap<String, EnvType>,
}
impl Env {
    pub fn new() -> Self {
        Self { variables: FxHashMap::default() }
    }

    pub fn std() -> Self {
        let mut o = Self::new();
        let f = [
            "*"
        ];
        for i in f {
            o.set(i.to_string(), EnvType::NativeProc(i.to_string()));
        }
        o.set("pi".to_string(), EnvType::Number(std::f64::consts::PI));
        o
    }

    pub fn native_call(name: &String, args: ArgStack) -> Result<EnvType, String> {
        match name.as_str() {
            "*" => {
                let [a, b] = &args[..] else { return Err("incorrect args".to_string()); };
                let a = match a {
                    EnvType::Number(a) => Ok(a),
                    _ => Err("not number".to_string())
                }?;
                let b = match b {
                    EnvType::Number(a) => Ok(a),
                    _ => Err("not number".to_string())
                }?;
                Ok(EnvType::Number(a * b))
            },
            _ => Err(format!("function {name} not found"))
        }
    }

    pub fn set(&mut self, name: String, val: EnvType) {
        self.variables.insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<EnvType> {
        self.variables.get(name).cloned()
    }
}