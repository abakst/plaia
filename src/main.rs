use plaia::language::plaia::concrete::*;
use plaia::language::plaia::signed::*;
use plaia_language::language::plaia::parse;

fn main() {
    let e = parse::parse_expression("3");
    println!("Eval {:?}", concrete_eval::<SimpleValue>(e));

    //let s2 = parse::parse_statement("{let tres : u64 = 3;\nlet ncuatro : u64; ncuatro = 1;\n}");
    let s2 = parse::parse_statement("{let tres : u64 = 3;\nlet ncuatro : u64; ncuatro = 1;\n}");
    concrete_run::<SimpleValue>(s2);

    let s3 = parse::parse_statement(
        "{
let x  : u64  = 0;
let t  : bool = true;
let px : u64* = &x;
*px = 13;
}",
    );

    // concrete_run::<SimpleValue>(s3);
    concrete_run::<SignedValue>(s3);
}
