use std::option::*;

pub type Loc = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
}

impl Symbol {
    pub fn new(nm: String) -> Symbol {
        Symbol { name: nm }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UnOp {
    Ref,
    Deref,

    Negate,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,

    And,
    Or,

    Proj,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    I64,
    Bool,
    Ptr(Box<Type>),
    Tuple(Vec<Type>),
    Vector(Box<Type>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LiteralKind {
    LBool(bool),
    LInt(i64),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Lit {
    pub lit: LiteralKind,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Lit(Lit),
    Var(Symbol),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    FunCall(Symbol, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub expr: ExprKind,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum LValKind {
    VarRef(Symbol),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum PatternKind {
    PWild,
    PSymbol(Symbol),
    PLiteral(Lit)
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern: PatternKind,
    pub loc: Loc
}

#[derive(Debug, Clone)]
pub enum CaseBranchKind {
    CaseArm(Pattern, Box<Statement>),
}

#[derive(Debug, Clone)]
pub struct CaseBranch {
    pub branch: CaseBranchKind,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Statement {
    pub stmt: StatementKind,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: Symbol,
    pub params: Vec<TypeBind>,
    pub body: Statement,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
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
