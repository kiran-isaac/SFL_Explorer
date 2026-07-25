use sfl_lib::{self as lib, typecheck};
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

    let mut main_expr = ast.get_assign_exp(match ast.get_main(ast.root) {
        Some(v) => v,
        None => {
            eprintln!("Main not found");
            std::process::exit(1);
        }
    });

    let mut rcs = lib::find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt);

    println!("{}", ast.to_string_sugar(main_expr, false));
    let mut i = 0;

    while let Some(rc) = rcs {
        let s1 = ast.to_string_sugar(rc.from, false);
        let s2 = rc.to.to_string_sugar(rc.to.root, false);
        println!("{i}: Next: {} => {}", s1, s2);

        // io::stdout().flush().unwrap();
        //
        // let mut input = String::new();
        // io::stdin()
        //     .read_line(&mut input)
        //     .expect("Failed to read line");
        // input = input.trim().to_string();

        main_expr = ast.get_assign_exp(match ast.get_main(ast.root) {
            Some(v) => v,
            None => {
                panic!("Main not found, should have been caught by parser");
            }
        });

        i += 1;

        ast.do_rc_subst(main_expr, &rc);

        rcs = lib::find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt);
        println!("\n{}", ast.to_string_sugar(main_expr, false));
    }
}
