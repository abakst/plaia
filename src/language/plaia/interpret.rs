use plaia_language::language::plaia::ast::*;
use std::collections::HashMap;

trait Evaluator {
    type V;
    type L;
    // V : Value
    // L : Location
    //
    // The store maps symbols (variables) to locations
    // The heap maps locations to values
    fn find_store(&mut self, s: &Symbol) -> Self::L;
    fn find_heap(&mut self, l: &Self::L) -> Self::V;
    fn update_store(&mut self, x: &Symbol, l: &Self::L);
    fn update_heap(&mut self, l: &Self::L, l: Self::V);
    fn alloc(&mut self) -> Self::L;

    // "Meaning"
    fn denote(&mut self, o: BinOp, e1: Self::V, e2: Self::V) -> Self::V;
    fn unit_int(&mut self, v:u64) -> Self::V;
    fn unit_loc(&mut self, l:Self::L) -> Self::V;
    fn unwrap_ptr(&mut self, v:Self::V) -> Self::L;
}

// A simple implementation:
pub type SimpleAddr = usize;

#[derive(Debug, Clone)]
pub enum SimpleValue {
    VInt(u64),
    VAddr(SimpleAddr)
}

type SimpleStore = HashMap<Symbol, SimpleAddr>;
struct SimpleEvaluator {
    heap: Vec<SimpleValue>,
    frames: Vec<SimpleStore>,
    cur_frame: usize
}

impl Evaluator for SimpleEvaluator {
    type V = SimpleValue;
    type L = SimpleAddr;

    fn find_store(&mut self, s: &Symbol) -> SimpleAddr {
        *self.frames[self.cur_frame].get(s).unwrap()
    }

    fn find_heap(&mut self, l: &SimpleAddr) -> SimpleValue {
        self.heap[*l].clone()
    }

    fn update_store(&mut self, s: &Symbol, l: &SimpleAddr) {
        self.frames[self.cur_frame].insert(s.clone(), *l);
    }

    fn update_heap(&mut self, l: &SimpleAddr, v: SimpleValue) {
        self.heap[*l] = v;
    }

    fn alloc(&mut self) -> SimpleAddr {
        self.heap.push(SimpleValue::VInt(0));
        self.heap.len() - 1
    }

    fn denote(&mut self, _o: BinOp, _v1: SimpleValue, _v2: SimpleValue) -> SimpleValue {
        panic!("Not implemented!")
    }

    fn unit_int(&mut self, v:u64) -> SimpleValue {
        SimpleValue::VInt(v)
    }
    fn unit_loc(&mut self, v: SimpleAddr) -> SimpleValue {
        SimpleValue::VAddr(v)
    }
    fn unwrap_ptr(&mut self, v:SimpleValue) -> SimpleAddr {
        if let SimpleValue::VAddr(addr) = v {
            addr
        } else {
            panic!("Not an address!")
        }
    }
}

fn eval_lval<E, K>(eval:&mut E, mut k:K, e:Expr) -> E::L
    where E: Evaluator,
          K: FnMut(&mut E, Expr) -> E::V {
    match e.expr {
        ExprKind::Var(x) => {
            eval.find_store(&x)
        }
        ExprKind::Deref(e) => {
            let v = k(eval, *e);
            eval.unwrap_ptr(v)
        }
        _ => panic!("Not an LVal!")
    }
}

fn eval_expr<E, K>(eval:&mut E, mut k:K, e:Expr) -> E::V
    where E: Evaluator,
          K: FnMut(&mut E, Expr) -> E::V

{
    match e.expr {
        ExprKind::Lit(v) => eval.unit_int(v),
        ExprKind::Var(x) => {
            let loc = eval.find_store(&x);
            eval.find_heap(&loc)
        }
        ExprKind::Binary(o, lhs, rhs) => {
            let v1 = k(eval, *lhs);
            let v2 = k(eval, *rhs);
            eval.denote(o, v1, v2)
        }
        ExprKind::Ref(e) => {
            let lv = eval_lval(eval, k, *e);
            eval.unit_loc(lv)
        }
        ExprKind::Deref(e) => {
            let vptr = k(eval, *e); // ptr should be a location
            let ptr  = eval.unwrap_ptr(vptr);
            eval.find_heap(&ptr)
        }
        _ => panic!("NotImplemented!")
    }
}

fn run_stmt<E, KE, KS>(eval: &mut E, mut ke:KE, mut ks:KS, s:Statement)
    where E: Evaluator,
          KE: FnMut(&mut E, Expr) -> E::V,
          KS: FnMut(&mut E, Statement)

{
    match s.stmt {
        StatementKind::Block(ss) => {
            for stmt in ss {
                ks(eval, stmt);
            }
        }
        StatementKind::Assign(lhs, rhs) => {
            let v   = ke(eval, rhs);
            let loc = eval_lval(eval, ke, lhs);
            eval.update_heap(&loc, v)
        }
        StatementKind::VarDecl(tb, oe) => {
            let l      = eval.alloc();
            eval.update_store(&tb.name, &l);

            if let Some(exp) = oe {
                let v = ke(eval, exp);
                eval.update_heap(&l, v);
            }
        }
    }
}

fn simple_cb(eval: &mut SimpleEvaluator, e:Expr) -> SimpleValue {
    eval_expr(eval, simple_cb, e)
}
fn simple_stmt_cb(eval: &mut SimpleEvaluator, s:Statement) {
    run_stmt(eval, simple_cb, simple_stmt_cb, s)
}

pub fn simple_eval(e:Expr) -> SimpleValue {
    let frame0   = HashMap::new();
    let mut eval = SimpleEvaluator { heap: Vec::new(), cur_frame: 0, frames: vec![frame0] };
    eval_expr(&mut eval, simple_cb, e)
}

pub fn simple_run(s:Statement) {
    let frame0   = HashMap::new();
    let mut eval = SimpleEvaluator { heap: Vec::new(), cur_frame: 0, frames: vec![frame0] };
    run_stmt(&mut eval, simple_cb, simple_stmt_cb, s);

    for (k,l) in &eval.frames[eval.cur_frame] {
        let v = eval.heap.get(*l);
        println!("{:?} => {:?}", k, v);
    }
}
