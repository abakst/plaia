use std::collections::HashMap;
use crate::language::plaia::interpret::*;
use plaia_language::language::plaia::ast::*;
use plaia_language::language::plaia::ast::{Statement};
// A simple concrete implementation:
pub type SimpleAddr = usize;

#[derive(Debug, Copy, Clone)]
pub enum SimpleValue {
    VInt(u64),
    VAddr(SimpleAddr),
}

type SimpleStore = HashMap<Symbol, SimpleAddr>;

#[derive(Debug, Clone)]
pub struct ConcreteEvaluator<Val> {
    heap: Vec<Val>,
    frames: Vec<SimpleStore>,
    cur_frame: usize,
    trace: Vec<(Vec<Val>, SimpleStore, Loc)>
}
type SimpleEvaluator = ConcreteEvaluator<SimpleValue>;

pub trait ValCompute<L> : Copy {
    fn zero() -> Self;
    fn op(o: &BinOp, e1: Self, e2: Self) -> Self;
    fn from_lit(l: &Lit) -> Self;
    fn from_loc(l: &L) -> Self;
    fn unwrap_loc(v: Self) -> L;
}

impl ValCompute<SimpleAddr> for SimpleValue {
    fn zero() -> Self
    {
        SimpleValue::VInt(0)
    }
    fn op(o: &BinOp, e1: Self, e2: Self) -> Self
    {
        match (e1, e2) {
            (SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                match o {
                    BinOp::Add => SimpleValue::VInt(v1 + v2),
                    BinOp::Sub => SimpleValue::VInt(v1 - v2),
                    BinOp::Mul => SimpleValue::VInt(v1 * v2),
                    BinOp::Div => SimpleValue::VInt(v1 / v2),
                }
            _ =>  panic!("Type Error")
        }
    }
    fn from_lit(l: &Lit) -> Self
    {
        match l.lit {
            LiteralKind::LInt(v) => SimpleValue::VInt(v),
            LiteralKind::LBool(true) => SimpleValue::VInt(1),
            LiteralKind::LBool(false) => SimpleValue::VInt(0),
        }
    }
    fn from_loc(l: &SimpleAddr) -> Self
    {
        SimpleValue::VAddr(*l)
    }

    fn unwrap_loc(v: Self) -> SimpleAddr
    {
        if let SimpleValue::VAddr(addr) = v {
            addr
        } else {
            panic!("Not an address!")
        }
    }
}

impl <Val> Evaluator for ConcreteEvaluator<Val> where Val : ValCompute<SimpleAddr> {
    type V = Val;
    type L = SimpleAddr;


    fn find_store<R,K>(&mut self, s: &Symbol, k: &K) -> R
    where K: Fn(&mut Self, Self::L) -> R
    {
        let l = *self.frames[0].get(s).unwrap();
        k (self, l)
    }

    fn find_heap<R,K>(&mut self, s: &Self::L, k: &K) -> R
    where K: Fn(&mut Self, Self::V) -> R
    {
        let v = self.heap[*s];
        k(self, v)
    }

    fn alloc<R,K>(&mut self, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R
    {
        self.heap.push(Val::zero());
        let l = self.heap.len() - 1;
        k(self, l)
    }

    fn update_store<K,R>(&mut self, x: &Symbol, l: &Self::L, k: &K) -> R
    where K: Fn(&mut Self) -> R
    {
        self.frames[self.cur_frame].insert(x.clone(), *l);
        k(self)
    }

    fn update_heap<K,R>(&mut self, l: &Self::L, v: Self::V, k: &K) -> R
    where K: Fn(&mut Self) -> R
    {
        self.heap[*l] = v;
        k(self)
    }

    // // "Meaning"
    fn denote<R,K>(&mut self, o: &BinOp, e1: Self::V, e2: Self::V, k: &K) -> R
        where K: Fn(&mut Self, Self::V) -> R
    {
        k(self, Val::op(o, e1, e2))

    }

    fn inj_val(&self, l: &Lit) -> Val {
        Val::from_lit(l)
    }

    fn inj_loc(&self, v: SimpleAddr) -> Val {
        Val::from_loc(&v)
    }

    fn unwrap_ptr(&self, v: Val) -> SimpleAddr {
        Val::unwrap_loc(v)
    }
}

pub fn concrete_stmt_cb<R>(eval: &mut SimpleEvaluator,
                           s: &Statement,
                           k: &dyn(Fn(&mut SimpleEvaluator) -> R)) -> R
{
    run_stmt(eval, s, &concrete_cb, &concrete_stmt_cb, k)
}

fn tracing_concrete_stmt_cb<Val:ValCompute<SimpleAddr>,R>(eval: &mut ConcreteEvaluator<Val>,
                                                          s: &Statement,
                                                          k: &dyn(Fn(&mut ConcreteEvaluator<Val>) -> R)) -> R
{
    let h  = eval.heap.clone();
    let st = eval.frames[eval.cur_frame].clone();
    let p = (h, st, s.loc);
    eval.trace.push(p);
    run_stmt(eval, s, &concrete_cb, &tracing_concrete_stmt_cb, k)
}

fn concrete_cb<Val:ValCompute<SimpleAddr>,R>(eval: &mut ConcreteEvaluator<Val>,
                                             e: &Expr,
                                             k: &dyn(Fn(&mut ConcreteEvaluator<Val>, Val) -> R)) -> R
{
    eval_expr(eval, e, &concrete_cb, k)
}


pub fn concrete_eval<Val:ValCompute<SimpleAddr>>(e: Expr) -> Val {
    let frame0 = HashMap::new();
    let mut eval = ConcreteEvaluator {
        heap: Vec::new(),
        cur_frame: 0,
        frames: vec![frame0],
        trace: Vec::new()
    };
    eval_expr(&mut eval, &e, &concrete_cb, &|_eval, v| v)
}

pub fn concrete_run<Val:ValCompute<SimpleAddr> + std::fmt::Debug>(s: Statement) {
    let frame0 = HashMap::new();
    let mut eval = ConcreteEvaluator::<Val> {
        heap: Vec::new(),
        cur_frame: 0,
        frames: vec![frame0],
        trace: Vec::new()
    };
    run_stmt(&mut eval, &s, &concrete_cb, &tracing_concrete_stmt_cb, &|_e| ());

    println!("Trace: ");
    for (h, st, loc) in &eval.trace {
        println!("State: {:?}", loc);
        for (k,l) in st {
            let v = h.get(*l);
            println!("\t{:?} => {:?}", k, v);
        }
    }
}
