use std::{cell::RefCell, any::Any};

use rustc_hash::FxHashMap;

use crate::Ast;

type ArgStack = Vec<EnvType>;

#[derive(Debug)]
struct ProcInfo<'a> {
    args: u8,
    body: &'a Ast,
    captured: EnvId, // need to make env have interior mutability
}

#[derive(Debug, Clone)]
pub enum EnvType {
    Number(f64),
    // Proc(String, ProcInfo<'a>),
    NativeProc(String),
    // List(Vec<EnvType>),
}

#[derive(Debug, Clone, Copy)]
pub struct EnvId(usize);

#[derive(Debug)]
pub struct EnvManager {
    parents: FxHashMap<usize, EnvId>,
    envs: FxHashMap<usize, Env>,
    counter: usize
}

impl EnvManager {
    pub fn new() -> Self {
        Self {
            parents: FxHashMap::default(),
            envs: FxHashMap::default(),
            counter: 0,
        }
    }

    pub fn parent(&self, id: &EnvId) -> Option<EnvId> {
        self.parents.get(&id.0).copied()
    }

    // find closest parent that contains variable name
    pub fn find_var(&self, id: &EnvId, name: &String) -> Option<EnvId> {
        if self.envs.get(&id.0).unwrap().contains(name) {
            Some(*id)
        } else {
            self.parent(id).and_then(|id| self.find_var(&id, name))
        }
    }

    pub fn get(&self, id: &EnvId) -> &Env {
        &self.envs[&id.0]
    }

    pub fn get_mut(&mut self, id: &EnvId) -> &mut Env {
        self.envs.get_mut(&id.0).unwrap()
    }

    pub fn new_env(&mut self, parent: Option<EnvId>) -> EnvId {
        let id = self.counter;
        let env = Env::new(EnvId(id), parent);
        self.envs.insert(id, env);
        self.counter += 1;
        EnvId(id)
    }

    pub fn std_env(&mut self) -> EnvId {
        let mut env = self.new_env(None);
        self.get_mut(&env).std();
        env
    }
}

#[derive(Debug)]
pub struct Env {
    id: EnvId,
    parent: Option<EnvId>,
    variables: FxHashMap<String, EnvType>,
}

impl Env {
    pub fn new(id: EnvId, parent: Option<EnvId>) -> Self {
        Self { id, parent, variables: FxHashMap::default() }
    }

    pub fn std(&mut self) {
        let f = [
            "*"
        ];
        for i in f {
            self.set(i.to_string(), EnvType::NativeProc(i.to_string()));
        }
        self.set("pi".to_string(), EnvType::Number(std::f64::consts::PI));
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

    pub fn contains(&self, name: &String) -> bool {
        self.variables.contains_key(name)
    }

    pub fn id(&self) -> EnvId {
        self.id
    }
}