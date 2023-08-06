use std::collections::VecDeque;

use env::{Env, EnvType};

mod env;

#[derive(Debug)]
enum Node {
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
}

#[derive(Debug)]
enum AstNode {
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

    fn to_var(env: &Env, n: &Node) -> Option<EnvType> {
        match n {
            Node::Symbol(s) => env.get(s),
            Node::Number(c) => Some(EnvType::Number(*c)),
        }
    }

    fn eval(ast: &Ast, env: &mut Env) -> Option<EnvType> {
        // resolve var if can no longer traverse
        if let AstNode::Leaf(n) = ast {
            let val = Self::to_var(env, n);
            if val.is_none() {
                // string literal not found in env
                panic!("instruction not found: {:?}", n);
            }
            println!("{:?}: {:?}", n, val);
            return val;
        }

        let body = ast.unwrap_body();

        // handle if first token is keyword
        let t = body.first().unwrap();
        if let AstNode::Leaf(n) = t {
            match n.unwrap_symbol().as_str() {
                "define" => {
                    let v = Self::eval(&body[2], env).unwrap();
                    env.set(body[1].unwrap_leaf().unwrap_symbol().clone(), v);
                    return None;
                },
                _ => {}
            }
        }

        let proc_ret = Self::eval(t, env);
        // library functions defined outside env
        if let Some(EnvType::NativeProc(name)) = proc_ret {
            let args = body[1..].iter().map(|a| Self::eval(a, env).unwrap()).collect();
            let ret = Env::native_call(&name, args);
            return Some(ret.unwrap());
        }

        // arbitrarily nested stuff
        return proc_ret;
    }
}

fn main() {
    let test = "(define r 10) (* pi (* r r))".to_string();
    let mut tokens = Parser::tokenise(test);
    println!("{tokens:?}");
    let ast = Parser::parse(&mut tokens);
    println!("{ast:#?}");

    let mut env = Env::std();
    for n in ast {
        let res = Parser::eval(&n, &mut env);
        println!("{:?}", res);
    }
}
