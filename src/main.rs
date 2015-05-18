extern crate brainfsk;
extern crate docopt;

use std::io;
use std::io::Read;
use std::fs::File;

const USAGE: &'static str = "
Usage: brainfsk [options] [--] <filename>
       brainfsk (-h | --help)

Options:
    -d, --dump  Instead of executing, dump string representation of ops to stdout
    -h, --help  Show this message
";

fn read_program(mut f: File) -> io::Result<String> {
    let mut program = String::new();
    try!(f.read_to_string(&mut program));
    Ok(program)
}

fn parse(s: String) -> Result<brainfsk::Program, String> {
    brainfsk::parse(&mut s.chars())
        .map_err(|e| format!("error while parsing: {}", e))
}

fn dump_tokens(p: &brainfsk::Program) {
    fn indent(depth: usize) -> String {
        let s = String::with_capacity(depth*4);
        (0..depth).map(|_| "     ")
            .fold(s, |mut r, s| { r.push_str(s); r })
    }

    fn dump(a: &brainfsk::Ast, depth: usize) {
        match *a {
            brainfsk::Op(ref x) =>
                println!("{}{:?}", indent(depth), x),
            brainfsk::Loop(ref x) if x.len() == 0 =>
                println!("{}Loop()", indent(depth)),
            brainfsk::Loop(ref x) => {
                print!("{}Loop(", indent(depth));
                dump(&x[0], 0);

                for node in x.iter().skip(1) {
                    dump(node, depth + 1)
                }
                println!("{}    )", indent(depth))
            }
        }
    }

    for node in p {
        dump(node, 0);
    }
}

/// take action based on program options
fn process_program(p: brainfsk::Program, args: docopt::ArgvMap) {
    if args.get_bool("--dump") {
        dump_tokens(&p)
    } else {
        let mut ctx = brainfsk::Context::new();
        match ctx.execute(&p) {
            Ok(_) => (),
            Err(x) => println!("error during execution: {}", x),
        };
    };
}

fn main() {
    let args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    let filename = args.get_str("<filename>").to_string();
    File::open(&filename)
        .and_then(read_program)
        .map_err(|e| format!("error reading {}: {}", filename, e))
        .and_then(parse)
        .map(|p| process_program(p, args))
        .unwrap_or_else(|e| println!("brainfsk: {}", e));
}
