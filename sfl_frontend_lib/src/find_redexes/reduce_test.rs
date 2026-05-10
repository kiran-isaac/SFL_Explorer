/// O(n^2) so only use for small things
#[allow(unused)]
fn assert_eq_in_any_order<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) {
    for x in a {
        let mut found = false;
        for y in b {
            if x == y {
                found = true;
            }
        }
        assert!(found);
    }
}

use crate::find_redexes::reduce::find_single_redex_contraction_pair;
use crate::{find_all_redex_contraction_pairs, typecheck, KnownTypeLabelTable, Parser};

#[test]
fn basic_add_test() {
    let program = "main :: Int\nmain = add 5 1";
    let ast = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap()
        .ast;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    let rcs =
        find_all_redex_contraction_pairs(&ast, Some(module), exp, &KnownTypeLabelTable::new());
    assert!(rcs.len() == 1);
    println!("{}", ast.rc_to_str(&rcs[0]));
}

#[test]
fn waits_for_eval() {
    let program = "func :: a -> a\nfunc x = x\nmain :: Int\nmain = func (add 5 1)";
    let pr = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rcs = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
    println!("{}", ast.rc_to_str(&rcs));
}

#[test]
fn myadd_test() {
    let program =
        "myadd::Int -> Int -> Int\nmyadd = \\x y.add x y\nmain::Int\nmain = myadd 2 3".to_string();

    let pr = Parser::from_string(program).parse_module(false).unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    let rc = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();

    assert_eq!("myadd 2 3 -> add 2 3", ast.rc_to_str(&rc));
}

#[test]
fn redexes_match() {
    let program = r#"
    data List a = Cons a (List a) | Nil

    main :: Bool
    main = match (Cons (5) Nil) {
      | Nil -> true
      | Cons _ _ -> false
    }"#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap();

    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rc = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
    assert_eq!("false", rc.to.to_string_sugar(rc.to.root, false));
}

#[test]
fn weird_halt_bug() {
    let program = r#"
    fac :: Int -> Int
    fac n = if (n <= 1) (1) (n * (fac (n - 1)))

    main :: Int
    main = fac 5"#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;

    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rcs = find_all_redex_contraction_pairs(&ast, Some(module), exp, &lt);
    assert_eq!(rcs.len(), 2);

    ast.do_rc_subst(exp, &rcs[1]);
    let rcs = find_all_redex_contraction_pairs(&ast, Some(module), exp, &lt);
    assert_eq!(rcs.len(), 1);
}
