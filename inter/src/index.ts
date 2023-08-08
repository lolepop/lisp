
function tokenise(input: string): string[] {
    const chars = [...input];
    const tokens: string[] = [];
    let acc = "";
    
    while (chars.length > 0) {
        const t = chars.shift();
        switch (t) {
            case "(":
            case ")":                
                if (acc.length > 0) {
                    tokens.push(acc);
                    acc = "";
                }
                tokens.push(t);
                break;
            case " ":
                if (acc.length > 0) {
                    tokens.push(acc);
                    acc = "";
                }
                break;
            default:
                acc += t;
        }
    }
    
    return tokens;
}

function parse(toks: string[]): Ast {
    const stack: Ast[] = [[]];
    let level = 0;
    
    while (toks.length > 0) {
        const t = toks.shift()!;
        
        switch (t) {
            case "(":
                stack.unshift([]);
                level++;
                break;
            case ")":
                const last = stack.shift()!;
                stack[0].push(last);
                level--;
                break;
            default:
                if (isNaN(parseFloat(t))) {
                    stack[0].push({ type: AstType.String, val: t });
                } else {
                    stack[0].push({ type: AstType.Float, val: parseFloat(t) });
                }
                break;
        }
    }
    
    if (level !== 0)
        throw new Error("mismatched brackets");
    return stack[0];
}

enum EnvType {
    String,
    Float,
    Bool,
    NativeProc,
    Proc
}
interface EnvVal { type: EnvType, val: any }

enum AstType {
    String,
    Float
}
interface AstVal { type: AstType, val: any }
type Ast = (AstVal | Ast)[];

class Env {
    parent?: Env;
    inner: { [k: string]: EnvVal } = {};

    constructor(parent: Env | undefined) {
        this.parent = parent;
    }

    public find(k: string): EnvVal | undefined {
        return this.inner[k] ?? this.parent?.[k];
    }

    public set(k: string, v: EnvVal) {
        this.inner[k] = v;
    }

    public static std(): Env {
        const env = this.default(undefined);
        Object.assign(env, {
            "pi": { type: EnvType.Float, val: 3.14 },
            "*": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ ...a, val: a.val * b.val }) },
            "/": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ ...a, val: a.val / b.val }) },
            "+": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ ...a, val: a.val + b.val }) },
            "-": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ ...a, val: a.val - b.val }) },
            "<=": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ type: EnvType.Bool, val: a.val <= b.val }) },
            "<": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ type: EnvType.Bool, val: a.val < b.val }) },
            ">=": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ type: EnvType.Bool, val: a.val >= b.val }) },
            ">": { type: EnvType.NativeProc, val: (a: EnvVal, b: EnvVal) => ({ type: EnvType.Bool, val: a.val >= b.val }) },
        });
        return env;
    }

    public static default(parent: Env | undefined): Env {
        const env = new Env(parent);
        const p = new Proxy(env, {
            get: (o, k) => k === "__inner" ? o : o.find(k.toString()),
            set: (o, k, v) => (o.set(k.toString(), v), true)
        });
        
        return p;
    }
}

function getVarOrConst(v: AstVal, env: Env): EnvVal | undefined {
    switch (v.type) {
        case AstType.Float:
            return { type: EnvType.Float, val: v.val };
        case AstType.String:
            return env[v.val];
    }
}

function exec(ast: Ast | AstVal, env: Env): EnvVal | undefined {
    // resolve var if can no longer traverse
    if (!(ast instanceof Array)) {
        const envVar = getVarOrConst(ast, env);
        if (!envVar) throw new Error(`instruction not found: ${ast}`);
        return envVar;
    }

    const t = ast[0];
    // peek first token is keyword
    if (!(t instanceof Array)) {
        if (t.val === "define") {
            env[(<AstVal>ast[1]).val] = exec(ast[2], env)!;
            return;
        } else if (t.val === "lambda") {
            return { type: EnvType.Proc, val: { args: (<Ast>ast[1]).map((a: AstVal) => a.val), closure: ast[2], captured: env } };
        } else if (t.val === "if") {
            const [condition, t, f] = ast.slice(1);
            return exec(exec(condition, env)!.val ? t : f, env);
        }
    }

    // first token does not match any keyword, assume proc call
    // first exec will recursively handle nested code (e.g. ((lambda (a) (a + 1)) 2) )
    const proc = exec(t, env);
    if (proc?.type === EnvType.NativeProc) {
        const args = ast.slice(1).map(arg => exec(arg, env));
        return proc.val.apply(null, args);
    } else if (proc?.type === EnvType.Proc) {
        const { args, closure, captured } = proc.val;
        const argVals = ast.slice(1).map(arg => exec(arg, env));

        // scope of the function being called, prevent caller args polluting other calls to same proc
        const scope = Env.default(captured);
        args.forEach((k, i) => { scope[k] = argVals[i]; });
        return exec(closure, scope);
    }

    return proc;
    
}

// const input = "(define r (lambda (arg1 arg2) (* arg1 arg2))) (r 3 2)";
// const input = "(define outer (lambda (a) (lambda (b) (* a b)))) ((outer 3) 2) ((outer 3) 3)";
const input = "(define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1)))))) (fact 100)";

const tokens = tokenise(input);
console.log(tokens);
const ast = parse(tokens);
console.log(JSON.stringify(ast));

const env = Env.std();
for (const i of ast) {
    const ret = exec(i as Ast, env);
    console.log(ret?.val);
}