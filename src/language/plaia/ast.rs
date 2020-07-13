use std::option::*;

type Loc = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    name : String,
}

impl Symbol {
    pub fn new(nm: String) -> Symbol {
        Symbol { name : nm }
    }
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Type {
    U64,
    Ptr(Type)
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(u64),
    Var(Symbol),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    FunCall(Symbol, Vec<Box<Expr>>)
}

#[derive(Debug)]
pub struct Expr {
    pub expr: ExprKind,
    pub loc: Loc
}

#[derive(Debug)]
pub enum LValKind {
    VarRef(Symbol)
}

#[derive(Debug)]
pub struct LVal {
    pub lval: LValKind,
    pub loc: Loc
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TypeBind {
    pub name: Symbol,
    pub bind_type: Type,
    pub loc: Loc
}

impl TypeBind {
    pub fn new(sym: Symbol, ty: Type, bindloc: Loc) -> TypeBind {
        TypeBind { name: sym, bind_type: ty, loc: bindloc }
    }
}

#[derive(Debug)]
pub enum StatementKind {
    VarDecl(TypeBind, Option<Expr>),
    Assign(LVal, Expr),
    Block(Vec<Box<Statement>>)
}

impl StatementKind {
    pub fn new_block(ss: Vec<Statement>) -> StatementKind {
        StatementKind::Block(ss.into_iter().map(|s| Box::new(s)).collect())
    }
}

#[derive(Debug)]
pub struct Statement {
    pub stmt: StatementKind ,
    pub loc: Loc
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Symbol,
    pub params: Vec<TypeBind>,
    pub body: Statement,
    pub loc: Loc
}

#[derive(Debug)]
pub struct Module {
    pub globals: Vec<TypeBind>,
    pub functions: Vec<FnDecl>,
    pub loc: Loc
}
/*
 * def x(a : t1, b : t2, ...) =  {
 *   let x : t = e...
 *   let y : u = e...
 * }
*/
