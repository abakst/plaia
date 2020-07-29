use plaia_language::language::plaia::ast::*;
use std::collections::HashMap;

trait Evaluator<A> {
    type V;
    type L;
    // V : Value
    // L : Location
    //
    // The store maps symbols (variables) to locations
    // The heap maps locations to values
    fn find_store(&mut self, s: &Symbol) -> dyn Evaluator<A, V = Self::V, L = Self::L>;

    fn find_heap(&mut self, l: &Self::L) -> Self::V;
    fn update_store(&mut self, x: &Symbol, l: &Self::L);
    fn update_heap(&mut self, l: &Self::L, l: Self::V);
    fn alloc(&mut self) -> Self::L;

    // "Meaning"
    fn denote(&mut self, o: BinOp, e1: Self::V, e2: Self::V) -> Self::V;
    fn inj_val(&mut self, v: Lit) -> Self::V;
    fn inj_loc(&mut self, l: Self::L) -> Self::V;
    fn unwrap_ptr(&mut self, v: Self::V) -> Self::L;
    fn bind<B, K>(k: K) -> dyn Evaluator<B, V = Self::V, L = Self::L>
    where K:  FnMut(Self, A) -> B;
}

// A simple implementation:
pub type SimpleAddr = usize;

#[derive(Debug, Clone)]
pub enum SimpleValue {
    VInt(u64),
    VAddr(SimpleAddr),
}

type SimpleStore = HashMap<Symbol, SimpleAddr>;
struct SimpleEvaluator<A> {
    answer: Option<A>,
    heap: Vec<SimpleValue>,
    frames: Vec<SimpleStore>,
    cur_frame: usize,
}

impl<A> Evaluator<A> for SimpleEvaluator<A> {
    type V = SimpleValue;
    type L = SimpleAddr;

    fn find_store(mut self, s: &Symbol) -> SimpleEvaluator<A>
    {
        let l = *self.frames[self.cur_frame].get(s).unwrap();
        self.answer = Some(l);
        self
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

    fn inj_val(&mut self, l: Lit) -> SimpleValue {
        match l.lit {
            LiteralKind::LInt(v) => SimpleValue::VInt(v),
            _ => panic!("ASDF"),
        }
    }

    fn inj_loc(&mut self, v: SimpleAddr) -> SimpleValue {
        SimpleValue::VAddr(v)
    }

    fn unwrap_ptr(&mut self, v: SimpleValue) -> SimpleAddr {
        if let SimpleValue::VAddr(addr) = v {
            addr
        } else {
            panic!("Not an address!")
        }
    }
}

fn eval_lval<E, K>(eval: &mut E, mut k: K, e: Expr) -> E::L
where
    E: Evaluator,
    K: FnMut(&mut E, Expr, fn(&mut E, E::V) -> E::L) -> E::L,
{
    match e.expr {
        ExprKind::Var(x) => eval.find_store(&x, |_eval, l| l),
        ExprKind::Deref(e) => {
            k(eval, *e, |eval1, v| eval1.unwrap_ptr(v))
        }
        _ => panic!("Not an LVal!"),
    }
}

fn eval_expr<B,K,E>(eval: E, k: K, e: Expr, bind: B) -> E::V
where
    E: Evaluator,
    K: FnMut(E, Expr, B) -> E::V,
    B: FnMut(E, E::V) -> E::V
{
    match e.expr {
        ExprKind::Lit(l) => eval.inj_val(l),
        ExprKind::Var(x) => eval.find_store(&x, |eval1, l| eval1.find_heap(&l)),
        ExprKind::Binary(o, lhs, rhs) => {
            k(eval, *lhs, |eval1: E, v1: E::V| {
                v1
                // k(eval1, *rhs, |eval2: E, v2| {
                //     eval2.denote(o, v1, v2)
                // })
            })
        }
        // ExprKind::Ref(e) => {
        //     let lv = eval_lval(eval, k, *e);
        //     eval.inj_loc(lv)
        // }
        // ExprKind::Deref(e) => {
        //     let vptr = k(eval, *e); // ptr should be a location
        //     let ptr = eval.unwrap_ptr(vptr);
        //     eval.find_heap(&ptr)
        // }
        _ => panic!("NotImplemented!"),
    }
}

fn run_stmt<E, KE, KS>(eval: &mut E, mut ke: KE, mut ks: KS, s: Statement)
where
    E: Evaluator,
    KE: FnMut(&mut E, Expr) -> E::V,
    KS: FnMut(&mut E, Statement),
{
    match s.stmt {
        StatementKind::Block(ss) => {
            for stmt in ss {
                ks(eval, stmt);
            }
        }
        StatementKind::Assign(lhs, rhs) => {
            let v = ke(eval, rhs);
            let loc = eval_lval(eval, ke, lhs);
            eval.update_heap(&loc, v)
        }
        StatementKind::VarDecl(tb, oe) => {
            let l = eval.alloc();
            eval.update_store(&tb.name, &l);

            if let Some(exp) = oe {
                let v = ke(eval, exp);
                eval.update_heap(&l, v);
            }
        }
        StatementKind::Case(discr, branches) => {
            let b = ke(eval, discr);
            for br in branches {}
            panic!("not implemented!")
        }
    }
}

fn simple_cb(eval: &mut SimpleEvaluator, e: Expr) -> SimpleValue {
    eval_expr(eval, simple_cb, e)
}
fn simple_stmt_cb(eval: &mut SimpleEvaluator, s: Statement) {
    run_stmt(eval, simple_cb, simple_stmt_cb, s)
}

pub fn simple_eval(e: Expr) -> SimpleValue {
    let frame0 = HashMap::new();
    let mut eval = SimpleEvaluator {
        heap: Vec::new(),
        cur_frame: 0,
        frames: vec![frame0],
    };
    eval_expr(&mut eval, simple_cb, e)
}

pub fn simple_run(s: Statement) {
    let frame0 = HashMap::new();
    let mut eval = SimpleEvaluator {
        heap: Vec::new(),
        cur_frame: 0,
        frames: vec![frame0],
    };
    run_stmt(&mut eval, simple_cb, simple_stmt_cb, s);

    for (k, l) in &eval.frames[eval.cur_frame] {
        let v = eval.heap.get(*l);
        println!("{:?} => {:?}", k, v);
    }
}
