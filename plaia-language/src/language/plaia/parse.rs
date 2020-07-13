use crate::language::plaia::ast::*;

lalrpop_mod!(pub plaia, "/language/plaia/plaia.rs");

pub fn parse_expression(inp: &str) -> Expr {
    plaia::ExprParser::new().parse(inp).unwrap()
}
pub fn parse_statement(inp: &str) -> Statement {
    plaia::StatementParser::new().parse(inp).unwrap()
}
pub fn parse_function(inp: &str) -> FnDecl {
    plaia::FnDeclParser::new().parse(inp).unwrap()
}
pub fn parse_module(inp: &str) -> Module {
    plaia::ModuleParser::new().parse(inp).unwrap()
}

#[test]
fn test_expr() {
    let e = parse_expression("3");
    if let ExprKind::Lit(l) = e.expr {
        assert!(l == 3);
    } else {
        assert!(false);
    }
}
