use super::*;

fn full_run_test(program: &str, typechecked: bool) -> String {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    if typechecked {
        typecheck(&mut ast, module, &mut lt, &tm).unwrap();
    }
    let mut main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());

    let mut rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    while rcs.len() != 0 {
        let rc = &rcs[0];
        ast.do_rc_subst(main_expr, rc);

        main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
        rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    }
    ast.to_string_sugar(main_expr, false)
}

#[test]
fn full_run_1() {
    let program = r#"
    x :: Int
    x = 5

    y :: Int
    y = 2

    inc :: Int -> Int
    inc = \i :: Int . add i 1

    main :: Int
    main = sub (add 5 (inc x)) (mul 5 y)
    "#;

    assert_eq!(full_run_test(program, true), "1");
}

#[test]
fn full_run_2() {
    let program = r#"
    x :: Int
    x = 100

    const_float::Int -> Float
    const_float = \_ :: Int. 1.5

    y :: Float
    y = const_float x

    inc :: Float -> Float
    inc = \i :: Float. addf i 1.0

    main :: Float
    main = inc y
    "#;

    assert_eq!(full_run_test(program, true), "2.5");
}

#[test]
fn untyped_y_combinator() {
    let program = r#"
    fac f n = if (n <= 1) (1) (n * (f (n - 1)))
    y f = (\x. f (x x)) (\x. f (x x))

    main = y fac 5"#;

    full_run_test(program, false);
}

#[test]
fn pairs_wtf() {
    let program = r#"
    type Coords = (Int, Int)

    add_scalar :: Int -> Coords -> Coords
    add_scalar s (x, y) = (2, 3)
    
    main :: Coords
    main = add_scalar 10 (2, 3)
    "#;

    full_run_test(program, true);
}
#[test]
fn pattern_match_sucks() {
    let program = r#"
    lenIsAtLeastTwo :: List a -> Bool
    lenIsAtLeastTwo list = match list {
        | Cons _ (Cons _ _) -> true
        | _ -> false
    }
    
    main :: Bool
    main = match ((Cons 1) (take 1 (infiniteFrom 2))) {
      | Cons _ (Cons _ _) -> true
      | _ -> false
    }
    "#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
    let rc = find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt).unwrap();
    println!("{:?}", rc.msg_before)
}

#[test]
fn pattern_match_sucks2() {
    let program = r#"
    lenIsAtLeastTwo list = match list {
        | Cons _ (Cons _ _) -> true
        | _ -> false
    }
    
    main = match (Cons 2 (infiniteFrom (2 + 1))) {
      | Nil -> Nil
      | Cons x xs -> if (1 > 0) (Cons x (take (1 - 1) xs)) Nil
    }
    
    /*
    match (Cons 1 (if (1 > 0) (Cons 2 (take (1 - 1) (infiniteFrom (2 + 1)))) Nil)) {
      | Cons _ (Cons _ _) -> true
      | _ -> false
    }*/
    "#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
    let rc = find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt).unwrap();
    println!("{:?}", rc.msg_before)
}

#[test]
fn pairs_again() {
    let program = r#"
data Suit = Hearts | Clubs | Spades | Diamonds

data Rank = Num Int | Jack | Queen | King | Ace

type Card = (Suit, Rank)

main :: Int
main = match (Diamonds, King) {
  | (_, Num n) 	-> n
  | (_, Jack) 	-> 11
  | (_, Queen) 	-> 12
  | (_, King) 	-> 13
  | (_, Ace) 		-> 14
}
    "#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
    let rc = find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt).unwrap();
    println!("{:?}", rc.msg_before)
}
