use crate::language::plaia::concrete::*;
use plaia_language::language::plaia::ast::*;


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SignedValue {
    VPos,
    VNeg,
    VZero,
    VUnknown,
    VAddr(SimpleAddr),
}

impl ValCompute<SimpleAddr> for SignedValue {
    fn zero() -> Self
    {
        SignedValue::VZero
    }

    fn op(o: &BinOp, e1: Self, e2: Self) -> Self {
        match (o, e1, e2) {
            (BinOp::Add, v1, v2) if v1 == v2    => v1,
            (BinOp::Add, SignedValue::VZero, x) => x,
            (BinOp::Add, x, SignedValue::VZero) => x,

            (BinOp::Sub, SignedValue::VZero, SignedValue::VPos) => SignedValue::VNeg,
            (BinOp::Sub, SignedValue::VZero, SignedValue::VNeg) => SignedValue::VPos,
            (BinOp::Sub, SignedValue::VZero, SignedValue::VZero) => SignedValue::VZero,
            (BinOp::Sub, SignedValue::VPos,  SignedValue::VNeg) => SignedValue::VPos,
            (BinOp::Sub, SignedValue::VNeg,  SignedValue::VPos) => SignedValue::VNeg,

            (BinOp::Mul, x, y) if x == Self::zero() || y == Self::zero() => SignedValue::VZero,
            (BinOp::Mul, SignedValue::VPos, x) => x,
            (BinOp::Mul, SignedValue::VNeg, SignedValue::VNeg) => SignedValue::VPos,
            (BinOp::Mul, SignedValue::VNeg, SignedValue::VPos) => SignedValue::VNeg,

            (BinOp::Div, _, SignedValue::VUnknown) => SignedValue::VUnknown,
            (BinOp::Div, SignedValue::VZero, _) => SignedValue::VZero,
            (BinOp::Div, x, SignedValue::VPos) => x,
            (BinOp::Div, SignedValue::VNeg, SignedValue::VNeg) => SignedValue::VPos,
            (BinOp::Div, SignedValue::VPos, SignedValue::VNeg) => SignedValue::VNeg,
            _ => SignedValue::VUnknown
        }
    }

    fn unwrap_loc(v: Self) -> SimpleAddr
    {
        if let SignedValue::VAddr(a) = v {
            a
        } else {
            panic!("Not an address!")
        }
    }

    fn from_loc(l: &SimpleAddr) -> Self
    {
        SignedValue::VAddr(*l)
    }

    fn from_lit(l: &Lit) -> Self
    {
        match l.lit {
            LiteralKind::LInt(v) if v > 0 => SignedValue::VPos,
            LiteralKind::LInt(_)          => SignedValue::VZero,
            LiteralKind::LBool(true)      => SignedValue::VPos,
            LiteralKind::LBool(false)     => SignedValue::VZero
        }
    }
}
