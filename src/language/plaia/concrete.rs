use std::collections::HashMap;
use crate::language::plaia::interpret::*;
use plaia_language::language::plaia::ast::*;
use plaia_language::language::plaia::ast::{Statement, FnDecl};
// A simple concrete implementation:
pub type SimpleAddr = usize;

#[derive(Debug, Clone)]
pub enum SimpleValue {
    VInt(i64),
    VAddr(SimpleAddr),
    VTuple(Vec<SimpleValue>),
    VVector(Vec<SimpleValue>),
}

type SimpleStore = HashMap<Symbol, SimpleAddr>;

#[derive(Debug, Clone)]
pub struct ConcreteEvaluator<Val> {
    heap: Vec<Val>,
    frames: Vec<SimpleStore>,
    cur_frame: usize,
    trace: Vec<(Vec<Val>, SimpleStore, Loc)>,
    decls: HashMap<Symbol, FnDecl>,
}
type SimpleEvaluator = ConcreteEvaluator<SimpleValue>;

pub trait ValCompute<L> : std::fmt::Debug + Clone {
    fn zero() -> Self;
    fn is_true(v: &Self) -> bool;
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
    fn is_true(v: &Self) -> bool
    {
        if let SimpleValue::VInt(x) = v {
            *x == 1
        } else {
            false
        }
    }
    fn op(o: &BinOp, e1: Self, e2: Self) -> Self
    {
        match (o, e1, e2) {
            (BinOp::Proj, SimpleValue::VTuple(vs), SimpleValue::VInt(v2)) =>
                vs[v2 as usize].clone(),
            (BinOp::Proj, SimpleValue::VVector(vs), SimpleValue::VInt(v2)) =>
                vs[v2 as usize].clone(),
            (BinOp::Add, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(v1 + v2),
            (BinOp::Sub, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(v1 - v2),
            (BinOp::Mul, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(v1 * v2),
            (BinOp::Div, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(v1 / v2),
            (BinOp::Eq, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 == v2 { 1 } else { 0 }),
            (BinOp::Neq, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 != v2 { 1 } else { 0 }),
            (BinOp::Lt, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 <  v2 { 1 } else { 0 }),
            (BinOp::Gt, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 >  v2 { 1 } else { 0 }),
            (BinOp::Lte, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 <= v2 { 1 } else { 0 }),
            (BinOp::Gte, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 >= v2 { 1 } else { 0 }),
            (BinOp::And, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 == 1 && v2 == 1 { 1 } else { 0 }),
            (BinOp::Or, SimpleValue::VInt(v1), SimpleValue::VInt(v2)) =>
                SimpleValue::VInt(if v1 == 1 || v2 == 1 { 1 } else { 0 }),
            _ => panic!("Type error")
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

impl <Val,R> Evaluator<R> for ConcreteEvaluator<Val> where Val : ValCompute<SimpleAddr> {
    type V = Val;
    type L = SimpleAddr;


    fn find_store<K:?Sized>(&mut self, s: &Symbol, k: &K) -> R
    where K: Fn(&mut Self, Self::L) -> R
    {
        let l = *self.frames[self.cur_frame].get(s).unwrap();
        k (self, l)
    }

    fn find_heap<K:?Sized>(&mut self, s: &Self::L, k: &K) -> R
    where K: Fn(&mut Self, Self::V) -> R
    {
        let v = self.heap[*s].clone();
        k(self, v)
    }

    fn alloc<K:?Sized>(&mut self, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R
    {
        self.heap.push(Val::zero());
        let l = self.heap.len() - 1;
        k(self, l)
    }

    fn update_store<K:?Sized>(&mut self, x: &Symbol, l: &Self::L, k: &K) -> R
    where K: Fn(&mut Self) -> R
    {
        self.frames[self.cur_frame].insert(x.clone(), *l);
        k(self)
    }

    fn update_heap<K:?Sized>(&mut self, l: &Self::L, v: Self::V, k: &K) -> R
    where K: Fn(&mut Self) -> R
    {
        self.heap[*l] = v;
        k(self)
    }

    fn push_frame<K:?Sized>(&mut self, frame: Vec<(Symbol, Self::V)>, k: &K) -> R
    where K: Fn(&mut Self) -> R
    {
        let mut map = HashMap::new();
        for (k,v) in frame {
            self.heap.push(v);
            map.insert(k,self.heap.len()-1);
        }

        self.frames.push(map);
        self.cur_frame += 1;
        k(self)
    }

    fn pop_frame(&mut self)
    {
        self.frames.pop();
        self.cur_frame -= 1;
    }

    fn return_loc(&mut self) -> Self::L
    {
        0
    }

    // "Meaning"
    fn denote<K:?Sized>(&mut self, o: &BinOp, e1: Self::V, e2: Self::V, k: &K) -> R
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

    fn do_match<K:?Sized>(&mut self, p: &PatternKind, v: &Self::V, k: &K) -> R
    where
        K: Fn(&mut Self, bool) -> R
    {
        match p {
            PatternKind::PLiteral(l) => {
                let lz = Self::V::from_lit(l);
                let b  = Self::V::op(&BinOp::Eq, lz, v.clone());
                k(self, Self::V::is_true(&b))
            }
            _ => todo!()
        }
    }

    fn fn_decl(&self, f: &Symbol) -> FnDecl
    {
        Clone::clone(self.decls.get(f).unwrap())
    }
}

pub fn concrete_stmt_cb<R>(eval: &mut SimpleEvaluator,
                           s: &Statement,
                           k: &(dyn Fn(&mut SimpleEvaluator) -> R)) -> R
{
    run_stmt(eval, s, &concrete_cb, &concrete_stmt_cb, k)
}

fn concrete_cb<R>(eval: &mut SimpleEvaluator,
                  e: &Expr,
                  k: &(dyn Fn(&mut SimpleEvaluator, SimpleValue) -> R)) -> R
{
    eval_expr(eval, e, &concrete_cb, &tracing_concrete_stmt_cb, k)
}

fn tracing_concrete_stmt_cb<R>(eval: &mut SimpleEvaluator,
                               s: &Statement,
                               k: &(dyn Fn(&mut SimpleEvaluator) -> R)) -> R
{
    let h  = eval.heap.clone();
    let st = eval.frames[eval.cur_frame].clone();
    let p = (h, st, s.loc);
    eval.trace.push(p);
    run_stmt(eval, s, &concrete_cb, &tracing_concrete_stmt_cb, k)
}



// pub fn concrete_eval<Val:ValCompute<SimpleAddr>>(e: Expr) -> Val {
//     let frame0 = HashMap::new();
//     let mut eval = ConcreteEvaluator::<Val> {
//         heap: Vec::new(),
//         cur_frame: 0,
//         frames: vec![frame0],
//         trace: Vec::new()
//     };
//     let ret : Box<dyn Fn(&mut ConcreteEvaluator<Val>, Val) -> Val> = Box::new(|_eval, v| v);
//     let stmt: &(dyn Fn(&mut E, &Statement, Box<dyn Fn(&mut )>))
//     eval_expr(&mut eval, &e, &concrete_cb, &tracing_concrete_stmt_cb, ret)
// }

pub fn initial_state(m: &Module, mut args: Vec<String>) -> (ConcreteEvaluator<SimpleValue>, FnDecl)
{
    // Traverse module and look for "main"
    let mut decls  = HashMap::new();
    let mut main   = None;
    for d in &m.functions {
        if d.name.name == "main" {
            main = Some(d.clone());
        } else {
            decls.insert(d.name.clone(), d.clone());
        }
    }

    if let Some(mainfn) = main {
        let cur_frame = 0;
        let iargs     = SimpleValue::VVector(args.iter_mut()
                                             .map(|s| SimpleValue::VInt(s.parse::<i64>().unwrap()))
                                             .collect());
        let mut frame = HashMap::new();
        frame.insert(mainfn.params[0].name.clone(), 1);

        let mut heap  = Vec::new();
        heap.push(SimpleValue::zero()); // Return value location
        heap.push(iargs);

        let frames    = vec![frame];
        let trace     = Vec::new();

        let eval = ConcreteEvaluator::<SimpleValue> {
            trace,
            frames,
            heap,
            cur_frame,
            decls,
        };
        (eval, mainfn)
    } else {
        panic!("No main function!")
    }
}


pub fn concrete_run(m: Module, args: Vec<String>, p: Option<&str>) {
    let (mut eval, f) = initial_state(&m, args);
    // let f        = eval.decls.get(&Symbol::new("main".to_string())).unwrap().body;

    run_stmt(&mut eval, &f.body, &concrete_cb, &tracing_concrete_stmt_cb, &|_e| ());

    let h  = eval.heap.clone();
    let st = eval.frames[eval.cur_frame].clone();
    eval.trace.push((h, st, f.loc));

    println!("Trace: ");
    for (h, st, (lo, hi)) in &eval.trace {
        for (k,l) in st {
            let v = h.get(*l);
            println!("\t{:?} => {:?}", k, v);
        }
        if let Some(pp) = p {
            println!("{}", &pp[*lo..*hi])
        }
    }
}
