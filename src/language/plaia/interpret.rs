use plaia_language::language::plaia::ast::*;
use plaia_language::language::plaia::ast::{Statement};

// fn foo<E>(e: E, x: &Symbol) -> impl (Fn(E) -> u64)
// where E: Eval<A, L = u64,V = u64>
// {
//     e.find(x, |loc| ({ move |_s| loc }))
// }

pub trait Evaluator
{
    type V : Copy;
    type L : Copy;
    // V : Value
    // L : Location
    //
    // The store maps symbols (variables) to locations
    // The heap maps locations to values

    fn find_store<R,K>(&mut self, s: &Symbol, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R;
    fn find_heap<R,K>(&mut self, s: &Self::L, k: &K) -> R
        where K: Fn(&mut Self, Self::V) -> R;

    fn alloc<R,K>(&mut self, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R;

    fn update_store<K,R>(&mut self, x: &Symbol, l: &Self::L, k: &K) -> R
        where K: Fn(&mut Self) -> R;

    fn update_heap<K,R>(&mut self, l: &Self::L, v: Self::V, k: &K) -> R
        where K: Fn(&mut Self) -> R;

    // // "Meaning"
    fn denote<R,K>(&mut self, o: &BinOp, e1: Self::V, e2: Self::V, k: &K) -> R
        where K: Fn(&mut Self, Self::V) -> R;
    fn do_match<R,K1,K2>(&mut self, p: &PatternKind, v: &Self::V, yes: &K1, no: &K2) -> R
    where
        K1: Fn(&mut Self) -> R,
        K2: Fn(&mut Self) -> R;

    fn inj_val(&self, v: &Lit) -> Self::V;
    fn inj_loc(&self, l: Self::L) -> Self::V;
    fn unwrap_ptr(&self, v: Self::V) -> Self::L;
}

fn eval_lval<E,Kont:?Sized,Rec:?Sized,R>(eval: &mut E, e: &Expr, r: &Rec, ret: &Kont) -> R
where
    E:    Evaluator,
    Rec:  Fn(&mut E, &Expr, &(dyn Fn(&mut E, E::V) -> R)) -> R,
    Kont: Fn(&mut E, E::L) -> R,
{
    match &e.expr {
        ExprKind::Var(x)   => eval.find_store(&x, &|e, l| ret(e, l)),
        ExprKind::Deref(e) => {
            r(eval, &e, &|eval1, ptrval| {
                let loc = eval1.unwrap_ptr(ptrval);
                ret(eval1, loc)
            })
        },
        _ => panic!("Not an LVal!"), //TODO: There should be a `fail` trait method..or continuation?
    }
}

pub fn eval_expr<E,Rec:?Sized,Kont:?Sized,R>(eval: &mut E, e: &Expr, r: &Rec, ret: &Kont) -> R
where
    E:    Evaluator,
    Rec:  Fn(&mut E, &Expr, &(dyn Fn(&mut E, E::V) -> R)) -> R,
    Kont: Fn(&mut E, E::V) -> R,
{
    match &e.expr {
        ExprKind::Lit(l) => {
            let v = eval.inj_val(l);
            ret(eval, v)
        },
        ExprKind::Var(x) => {
            eval.find_store(&x, &|eval: &mut E, l:E::L| {
                eval.find_heap(&l, &|eval, val| {
                    ret(eval, val)
                })
            })
        },
        ExprKind::Binary(o, lhs, rhs) => {
            let with_e1 = &|eval: &mut E, lhsval| {
                r(eval, &rhs, &|eval: &mut E, rhsval| {
                    eval.denote(&o, lhsval, rhsval, &|eval, v| ret(eval, v))
                })
            };
            r(eval, &lhs, with_e1)
        },
        ExprKind::Ref(e) => {
            eval_lval(eval, e, r, &|eval1: &mut E, loc: E::L| {
                let ptr = eval1.inj_loc(loc);
                ret(eval1, ptr)
            })
        }
        ExprKind::Deref(e) => {
            let k = &|eval: &mut E, ptr:E::V| {
                //  This continuation will get the result of calling `eval` on e.
                //  This should be a pointer, so unwrap it:
                let loc = &eval.unwrap_ptr(ptr);
                // Now get the value at the unwrapped location
                eval.find_heap(loc, &|eval: &mut E, val:E::V| {
                    ret(eval, val)
                })
            };
            r(eval, &e, k)
        }
        _ => todo!("eval_expr!"),
    }
}

pub fn run_stmt<E,Kont:?Sized,EvalRec:?Sized,StmtRec:?Sized,R>(eval: &mut E, s: &Statement, expr_rec: &EvalRec, rec: &StmtRec, ret: &Kont) -> R
where
    E: Evaluator,
    Kont: Fn(&mut E) -> R,
    EvalRec: Fn(&mut E, &Expr, &(dyn Fn(&mut E, E::V) -> R)) -> R,
    StmtRec: Fn(&mut E, &Statement, &(dyn Fn(&mut E) -> R)) -> R,
{
    match &s.stmt {
        StatementKind::Block(ss) => {
            if ss.is_empty() {
                ret(eval)
            } else {
                let base : Box<(dyn Fn(&mut E) -> R)>
                    = Box::new(|e: &mut E| { ret(e) });

                ss.iter().rev().fold(base, |acc, s| {
                    Box::new(move |e: &mut E| {
                        rec(e, &s, &|e: &mut E| {
                            acc(e)
                        })
                    })
                })(eval)
            }
        }

        StatementKind::Assign(lhs, rhs) => {
            expr_rec(eval, &rhs, &|eval: &mut E, val: E::V| {
                eval_lval(eval, &lhs, expr_rec, &|eval: &mut E, lval: E::L| {
                    eval.update_heap(&lval, val, &|eval: &mut E| ret(eval))
                })
            })
        }
        StatementKind::VarDecl(tb, Some(exp)) => {
            expr_rec(eval, &exp, &|eval, value| {
                eval.alloc(&|eval: &mut E, loc: E::L| {
                    eval.update_store(&tb.name, &loc, &|eval| {
                        eval.update_heap(&loc, value, &|eval| ret(eval))
                    })
                })
            })
        }
        StatementKind::VarDecl(tb, _) => {
            eval.alloc(&|eval: &mut E, loc: E::L| {
                eval.update_store(&tb.name, &loc, &|eval| ret(eval))
            })
        }
        StatementKind::Case(discr, branches) => {
            expr_rec(eval, discr, &|eval, discrval| {
                let base : Box<(dyn Fn(&mut E) -> R)>
                    = Box::new(|e: &mut E| { ret(e) });

                branches.iter().rev().fold(base, |acc, branch| {
                    let CaseBranchKind::CaseArm(pat, stmt) = &branch.branch;
                    Box::new(move |e: &mut E| {
                        e.do_match(&pat.pattern,
                                      &discrval,
                                      &|eval: &mut E| rec(eval, stmt, &|e: &mut E| { ret(e) }),
                                      &|eval: &mut E| acc(eval))
                    })
                })(eval)
            })
        }
        _ => todo!()
    }
}
