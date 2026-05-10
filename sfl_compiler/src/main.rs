use sfl_frontend_lib::{self as lib, typecheck};
use std::{env, fs};

static HORIZONTAL_SEPARATOR: &str =
    "______________________________________________________________";

fn main() {
    let argv: Vec<String> = env::args().collect();

    let file_path = if argv.len() == 2 {
        argv[1].clone()
    } else {
        eprintln!("Incorrect args");
        std::process::exit(1);
    };

    let file_string = if fs::metadata(&file_path).is_ok() {
        fs::read_to_string(&file_path).expect("Failed to read file")
    } else {
        eprintln!("File does not exist: {}", file_path);
        std::process::exit(1);
    };

    let pr = lib::Parser::from_string(file_string).parse_module(true);
    if let Err(e) = &pr {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    let pr = pr.unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    // Typecheck
    let module = ast.root;

    typecheck(&mut ast, module, &mut lt, &tm).unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1)
    });

    println!(
        "Typed: \n{}\n{}\n",
        ast.to_string_sugar(ast.root, true),
        HORIZONTAL_SEPARATOR
    );
}
