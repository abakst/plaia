use plaia_language::language::plaia::ast::*;
use plaia_language::language::plaia::ast::{Statement};

pub trait Evaluator<R>
{
    type V : Clone;
    type L : Clone;
    // V : Value
    // L : Location
    //
    // The store maps symbols (variables) to locations
    // The heap maps locations to values
    fn find_store<K:?Sized>(&mut self, s: &Symbol, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R;
    fn find_heap<K:?Sized>(&mut self, s: &Self::L, k: &K) -> R
        where K: Fn(&mut Self, Self::V) -> R;

    fn alloc<K:?Sized>(&mut self, k: &K) -> R
        where K: Fn(&mut Self, Self::L) -> R;
    fn update_store<K:?Sized>(&mut self, x: &Symbol, l: &Self::L, k: &K) -> R
        where K: Fn(&mut Self) -> R;
    fn update_heap<K:?Sized>(&mut self, l: &Self::L, v: Self::V, k: &K) -> R
        where K: Fn(&mut Self) -> R;

    fn push_frame<K:?Sized>(&mut self, frame: Vec<(Symbol, Self::V)>, k: &K) -> R
    where K: Fn(&mut Self) -> R;
    fn pop_frame(&mut self);
    fn return_loc(&mut self) -> Self::L;

    // // "Meaning"
    fn denote<K:?Sized>(&mut self, o: &BinOp, e1: Self::V, e2: Self::V, k: &K) -> R
        where K: Fn(&mut Self, Self::V) -> R;
    fn do_match<K:?Sized>(&mut self, p: &PatternKind, v: &Self::V, k: &K) -> R
    where
        K: Fn(&mut Self, bool) -> R;

    fn inj_val(&self, v: &Lit) -> Self::V;
    fn inj_loc(&self, l: Self::L) -> Self::V;
    fn unwrap_ptr(&self, v: Self::V) -> Self::L;

    fn fn_decl(&self, f: &Symbol) -> FnDecl;
}

fn eval_lval<E,Kont,Rec:?Sized,R>(eval: &mut E, e: & Expr, r: & Rec, ret: &Kont) -> R
where
    E:    Evaluator<R>,
    Rec:  Fn(&mut E, & Expr, &(dyn Fn(&mut E, E::V) -> R)) -> R,
    Kont: ?Sized + Fn(&mut E, E::L) -> R,
{
    match &e.expr {
        ExprKind::Var(x)   => eval.find_store(&x, ret),
        ExprKind::Unary(UnOp::Deref, e) => {
            r(eval, &e, &|eval1: &mut E, ptrval| {
                let loc = eval1.unwrap_ptr(ptrval);
                ret(eval1, loc)
            })
        },
        _ => panic!("Not an LVal!"), //TODO: There should be a `fail` trait method..or continuation?
    }
}

pub fn eval_expr<E,StmtRec:?Sized,Rec:?Sized,Kont,R>(eval: &mut E, e: & Expr, r: & Rec, stmt_rec: & StmtRec, ret: & Kont) -> R
where
    E:    Evaluator<R>,
    Rec:  Fn(&mut E, & Expr, &(dyn Fn(&mut E, E::V) -> R)) -> R,
    StmtRec: Fn(&mut E, &Statement, &(dyn Fn(&mut E) -> R)) -> R,
    Kont: ?Sized + Fn(&mut E, E::V) -> R,
{
    match &e.expr {
        ExprKind::Lit(l) => {
            let v = eval.inj_val(l);
            ret(eval, v)
        },
        ExprKind::Var(x) => {
            eval.find_store(&x, &|eval: &mut E, l:E::L| {
                eval.find_heap(&l, ret)
            })
        },
        ExprKind::Binary(o, lhs, rhs) => {
            let with_e1 = &move |eval: &mut E, lhsval: E::V| {
                r(eval, &rhs, &move |eval: &mut E, rhsval: E::V| {
                    eval.denote(&o, lhsval.clone(), rhsval, ret)
                })
            };
            r(eval, &lhs, with_e1)
        },
        ExprKind::Unary(UnOp::Ref, e) => {
            eval_lval(eval, e, r, &|eval1: &mut E, loc: E::L| {
                let ptr = eval1.inj_loc(loc);
                ret(eval1, ptr)
            })
        }
        ExprKind::Unary(UnOp::Deref, e) => {
            let k = &|eval: &mut E, ptr:E::V| {
                //  This continuation will get the result of calling `eval` on e.
                //  This should be a pointer, so unwrap it:
                let loc = &eval.unwrap_ptr(ptr);
                // Now get the value at the unwrapped location
                eval.find_heap(loc, ret)
            };
            r(eval, &e, k)
        }
        ExprKind::Unary(_op, _e) => todo!(),
        ExprKind::FunCall(f, es) => {
            let decl = eval.fn_decl(f);

            let base : Box<(dyn Fn(&mut E, Vec<E::V>) -> R)>
                = Box::new(move |e: &mut E, vs: Vec<E::V>| {
                    // push args
                    let in_frame : &(dyn Fn(&mut E) -> R)
                        = &|e| {
                            let body = Clone::clone(&decl.body);
                            let with_stmt : &(dyn Fn(&mut E) -> R) = &|e| {
                                e.pop_frame();
                                let l = e.return_loc();
                                e.find_heap(&l, ret)
                            };
                            stmt_rec(e, &body, with_stmt)
                        };
                    let args = decl.params
                                   .iter()
                                   .map(|tb| tb.name.clone())
                                   .zip(vs)
                                   .collect(); // TODO: Don't really need to do this?
                    e.push_frame(args, in_frame)
                });

            let call_with_args = es.iter().rev().fold(base, |acc, arg_exp| {
                Box::new(move |e, args| {
                    r(e, arg_exp, &|e, v| {
                        let mut args2 = args.clone();
                        args2.push(v);
                        acc(e, args2)
                    })
                })
            });

            call_with_args(eval, Vec::new())
        }
    }
}

pub fn run_stmt<E,Kont:?Sized,EvalRec:?Sized,StmtRec:?Sized,R>(eval: &mut E, s: & Statement, expr_rec: & EvalRec, rec: & StmtRec, ret: &Kont) -> R
where
    E: Evaluator<R>,
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
            expr_rec(eval, &rhs, &move |eval: &mut E, val: E::V| {
                eval_lval(eval, &lhs, expr_rec, &move |eval, lval| eval.update_heap(&lval, val.clone(), ret))
            })
        }

        StatementKind::VarDecl(tb, Some(exp)) => {
            expr_rec(eval, &exp, &move |eval, value| {
                eval.alloc(&|eval: &mut E, loc: E::L| {
                    eval.update_store(&tb.name, &loc, &|eval: &mut E| {
                        eval.update_heap(&loc, value.clone(), ret)
                    })
                })
            })
        }

        StatementKind::VarDecl(tb, _) => {
            eval.alloc(&|eval: &mut E, loc: E::L| {
                eval.update_store(&tb.name, &loc, ret)
            })
        }

        StatementKind::Case(discr, branches) => {
            expr_rec(eval, discr, &move |e, discrval| {
                let base : Box<(dyn Fn(&mut E, bool) -> R)>
                    = Box::new(|e: &mut E, _b: bool| { ret(e) });

                branches.iter().fold(base, |acc, branch| {
                    let CaseBranchKind::CaseArm(pat, stmt) = &branch.branch;
                    let v = discrval.clone();
                    Box::new(move |e, done| {
                        if done {
                            acc(e, done)
                        } else {
                            e.do_match(&pat.pattern, &v, &|e, b| {
                                    if b {
                                        rec(e, stmt, &|e| { acc(e, true) })
                                    } else {
                                        acc(e, false)
                                    }
                            })
                        }
                    })
                })(e, false)
            })
        }
    }
}
