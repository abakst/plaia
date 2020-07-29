use std::option::*;

type Loc = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    name: String,
}

impl Symbol {
    pub fn new(nm: String) -> Symbol {
        Symbol { name: nm }
    }
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Type {
    U64,
    Bool,
    Ptr(Box<Type>),
}

#[derive(Debug)]
pub enum LiteralKind {
    LBool(bool),
    LInt(u64),
}

#[derive(Debug)]
pub struct Lit {
    pub lit: LiteralKind,
    pub loc: Loc,
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(Lit),
    Var(Symbol),
    Ref(Box<Expr>),
    Deref(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    FunCall(Symbol, Vec<Expr>),
}

#[derive(Debug)]
pub struct Expr {
    pub expr: ExprKind,
    pub loc: Loc,
}

#[derive(Debug)]
pub enum LValKind {
    VarRef(Symbol),
}

#[derive(Debug)]
pub struct LVal {
    pub lval: LValKind,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct TypeBind {
    pub name: Symbol,
    pub bind_type: Type,
    pub loc: Loc,
}

impl TypeBind {
    pub fn new(sym: Symbol, ty: Type, bindloc: Loc) -> TypeBind {
        TypeBind {
            name: sym,
            bind_type: ty,
            loc: bindloc,
        }
    }
}

#[derive(Debug)]
pub enum PatternKind {
    PWild,
    PSymbol(Symbol),
    PLiteral(Lit)
}

#[derive(Debug)]
pub struct Pattern {
    pub pattern: PatternKind,
    pub loc: Loc
}

#[derive(Debug)]
pub enum CaseBranchKind {
    CaseArm(Pattern, Box<Statement>),
}

#[derive(Debug)]
pub struct CaseBranch {
    pub branch: CaseBranchKind,
    pub loc: Loc,
}

#[derive(Debug)]
pub enum StatementKind {
    VarDecl(TypeBind, Option<Expr>),
    Assign(Expr, Expr),
    Block(Vec<Statement>),
    Case(Expr, Vec<CaseBranch>),
}

pub fn if_statement(e: Expr, s: Statement) -> StatementKind {
    let l = s.loc;
    let tt = PatternKind::PLiteral(Lit {
        lit: LiteralKind::LBool(true),
        loc: l,
    });
    let pat = Pattern {
        pattern: tt,
            loc: s.loc
    };
    let branch = CaseBranchKind::CaseArm(pat, Box::new(s));
    let then_branch = CaseBranch {
        branch,
        loc: l,
    };
    StatementKind::Case(e, vec![then_branch])
}

impl StatementKind {
    pub fn new_block(ss: Vec<Statement>) -> StatementKind {
        StatementKind::Block(ss)
    }
}

#[derive(Debug)]
pub struct Statement {
    pub stmt: StatementKind,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Symbol,
    pub params: Vec<TypeBind>,
    pub body: Statement,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct Module {
    pub globals: Vec<TypeBind>,
    pub functions: Vec<FnDecl>,
    pub loc: Loc,
}
/*
 * def x(a : t1, b : t2, ...) =  {
 *   let x : t = e...
 *   let y : u = e...
 * }
*/
