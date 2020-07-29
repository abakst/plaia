use plaia::language::plaia::interpret;
use plaia_language::language::plaia::parse;

fn main() {
    let e = parse::parse_expression("3");
    println!("Eval {:?}", interpret::simple_eval(e));

    let s2 = parse::parse_statement("{let tres : u64 = 3;\nlet ncuatro : u64; ncuatro = 1;\n}");
    interpret::simple_run(s2);

    let s3 = parse::parse_statement(
        "{
let x  : u64  = 0;
let t  : bool = true;
let px : u64* = &x;
*px = 1;
if t then
  *px = 4;
}",
    );

    interpret::simple_run(s3);
}
