use std::collections::VecDeque;

use env::{Env, EnvType, EnvId, EnvManager, ProcInfo};

mod env;

#[derive(Debug)]
pub enum Node {
    Symbol(String),
    Number(f64),
}
impl Node {
    fn unwrap_symbol(&self) -> &String {
        match self {
            Node::Symbol(a) => a,
            _ => panic!("node is constant"),
        }
    }

    fn resolve<'a>(&self, env_manager: &EnvManager<'a>, env_id: &EnvId) -> Option<EnvType<'a>> {
        match self {
            Node::Symbol(s) => env_manager.find_var(env_id, s).and_then(|id| env_manager.get(&id).get(s)),
            Node::Number(c) => Some(EnvType::Number(*c)),
        }
    }
}

#[derive(Debug)]
pub enum AstNode {
    Leaf(Node),
    Body(Vec<AstNode>),
}
impl AstNode {
    fn push(&mut self, a: AstNode) {
        match self {
            AstNode::Body(n) => n.push(a),
            _ => panic!("no")
        }
    }

    fn unwrap_leaf(&self) -> &Node {
        match self {
            AstNode::Leaf(a) => a,
            _ => panic!("astnode is body"),
        }
    }

    fn unwrap_body(&self) -> &Vec<AstNode> {
        match self {
            AstNode::Body(a) => a,
            _ => panic!("astnode is leaf"),
        }
    }
}

type Ast = AstNode;
type Tokens = VecDeque<String>;

struct Parser {}
impl Parser {
    fn tokenise(s: String) -> Tokens {
        let mut tokens = VecDeque::new();

        let mut chars = s.chars();
        let mut acc = String::new();
        while let Some(c) = chars.next() {
            match c {
                '(' | ')' => {
                    if acc.len() > 0 {
                        tokens.push_back(acc);
                        acc = String::new();
                    }
                    tokens.push_back(c.to_string());
                }
                ' ' => {
                    if acc.len() > 0 {
                        tokens.push_back(acc);
                        acc = String::new();
                    }
                }
                c => acc.push(c),
            }
        }

        tokens
    }

    fn parse(tokens: &mut Tokens) -> Vec<Ast> {
        let mut level = 0;

        let mut stack = VecDeque::from([AstNode::Body(Vec::new())]);
        while tokens.len() > 0 {
            let tok = tokens.pop_front().unwrap();

            match tok.as_str() {
                "(" => {
                    // create new nesting
                    let ast = AstNode::Body(Vec::new());
                    stack.push_front(ast);
                    level += 1;
                }
                ")" => {
                    // join into previous nested
                    let n = stack.pop_front().unwrap();
                    stack.front_mut().unwrap().push(n);
                    level -= 1;
                }
                _ => {
                    let t = tok.parse::<f64>().map_or_else(|_| Node::Symbol(tok), |a| Node::Number(a));
                    stack.front_mut().unwrap().push(AstNode::Leaf(t))
                },
            }
        }

        // println!("{stack:#?}");

        assert!(level == 0);

        match stack.pop_front().unwrap() {
            AstNode::Body(a) => a,
            _ => { panic!("no"); }
        }
        // todo!()
    }

    fn eval<'a>(ast: &'a Ast, env_manager: &mut EnvManager<'a>, env_id: EnvId) -> Option<EnvType<'a>> {
        // let env = env_manager.env(&env_id);

        // resolve var if can no longer traverse
        if let AstNode::Leaf(n) = ast {
            let val = n.resolve(env_manager, &env_id);
            if val.is_none() {
                // string literal not found in env
                // println!("{env_manager:#?}");
                panic!("instruction not found: {:?}", n);
            }
            println!("{:?}: {:?}", n, val);
            return val;
        }

        let body = ast.unwrap_body();

        // handle if first token is keyword
        let t = body.first().unwrap();
        if let AstNode::Leaf(_n @ Node::Symbol(n)) = t {
            match n.as_str() {
                "define" => {
                    let v = Self::eval(&body[2], env_manager, env_id).unwrap();
                    let env = env_manager.get_mut(&env_id);
                    env.set(body[1].unwrap_leaf().unwrap_symbol().clone(), v);
                    return None;
                },
                "lambda" => {
                    let args = body[1].unwrap_body().iter().map(|arg| arg.unwrap_leaf().unwrap_symbol().clone()).collect();
                    return Some(EnvType::Proc(ProcInfo::new(args, &body[2], env_id)));
                }
                _ => {}
            }
        }

        let proc_ret = Self::eval(t, env_manager, env_id);
        // library functions defined outside env
        if let Some(EnvType::NativeProc(name)) = proc_ret {
            let args = body[1..].iter().map(|a| Self::eval(a, env_manager, env_id).unwrap()).collect();
            let ret = Env::native_call(&name, args);
            return Some(ret.unwrap());
        } else if let Some(EnvType::Proc(proc)) = proc_ret {
            let scope = env_manager.new_env(Some(proc.captured()));
            // no lazy eval :(
            let arg_vals = body.iter().skip(1).map(|arg| Self::eval(arg, env_manager, env_id).unwrap()).collect::<Vec<_>>();

            let env = env_manager.get_mut(&scope);
            for (k, v) in proc.args().iter().zip(arg_vals) {
                env.set(k.clone(), v);
            }
            
            let ret = Self::eval(proc.body(), env_manager, scope);
            return Some(ret.unwrap());
        }

        // arbitrarily nested stuff
        return proc_ret;
    }
}

fn main() {
    let test = "(define outer (lambda (a) (lambda (b) (* a b)))) ((outer 3) 2) ((outer 3) 3)".to_string();
    let mut tokens = Parser::tokenise(test);
    // println!("{tokens:?}");
    let ast = Parser::parse(&mut tokens);
    // println!("{ast:#?}");

    let mut env_manager = EnvManager::new();
    let root_env = env_manager.std_env();
    for n in &ast {
        let res = Parser::eval(n, &mut env_manager, root_env);
        println!("{:?}", res);
    }
}
