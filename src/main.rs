use plaia::language::plaia::concrete::*;
use plaia::language::plaia::signed::*;
use plaia_language::language::plaia::parse;

use std::env;
use std::fs::read_to_string;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let _prog  = args.remove(0);
    let file = args.remove(0);

    let contents = read_to_string(file)
        .expect("Something went wrong reading the file");

    let prog = parse::parse_module(&contents);

    concrete_run(prog, args, Some(&contents));
    // // let e = parse::parse_expression("3");
    // // println!("Eval {:?}", concrete_eval::<SimpleValue>(e));

    // // let s2 = parse::parse_statement("{let tres : i64 = 3;\nlet cuatro : i64; cuatro = 1;\n}");
    // // concrete_run::<SimpleValue>(s2, None);

    // // let s  = "{\n\tlet x  : i64  = 0;\n\tlet t  : bool = true;\n\tlet px : i64* = &x;\n\t*px = *px - 13;\n}";
    // // let s3 = parse::parse_statement(s);

    // // concrete_run::<SimpleValue>(s3, Some(s));
    // //
    // let s = "if y < 0 then { y = 1; }";
    // let pf = parse::parse_statement(s);
    // println!("f: {:?}", pf);

    // let f = "def main(args : vec<i64>) = { let x : i64; let y : i64; let z : i64; x = 1; y = 2; if ((y > 0) && (x > 0)) then { if ((x < 10) && (y < 3)) then { z = x + y; } } }";
    // let pf = parse::parse_function(f);
    // concrete_run::<SimpleValue>(pf, Some(f));
}
