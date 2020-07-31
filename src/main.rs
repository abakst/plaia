use plaia::language::plaia::concrete::*;
use plaia::language::plaia::signed::*;
use plaia_language::language::plaia::parse;

fn main() {
    let e = parse::parse_expression("3");
    println!("Eval {:?}", concrete_eval::<SimpleValue>(e));

    let s2 = parse::parse_statement("{let tres : i64 = 3;\nlet cuatro : i64; cuatro = 1;\n}");
    concrete_run::<SimpleValue>(s2, None);

    let s  = "{\n\tlet x  : i64  = 0;\n\tlet t  : bool = true;\n\tlet px : i64* = &x;\n\t*px = *px - 13;\n}";
    let s3 = parse::parse_statement(s);

    concrete_run::<SimpleValue>(s3, Some(s));
}
