use super::*;
use crate::Parser;

fn tc_test_should_pass(program: &str) -> Result<(), TypeError> {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm)
}

fn tc_test_should_pass_no_prelude(program: &str) -> Result<(), TypeError> {
    let pr = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm)
}

fn tc_test_should_fail(program: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm).unwrap_err();
}

#[test]
fn type_check_int_assign() -> Result<(), TypeError> {
    tc_test_should_pass("x :: Int\nx=10\nmain :: Int\nmain = x")
}

#[test]
fn type_check_int_add() -> Result<(), TypeError> {
    tc_test_should_pass("main :: Int\nmain = add 2 2")
}

#[test]
fn type_check_int_add_fail() {
    tc_test_should_fail("main :: Int -> Int\nmain = add 2 2")
}

#[test]
fn type_check_eq() -> Result<(), TypeError> {
    tc_test_should_pass("main :: Bool\nmain = eq (add 2 2) (mul 2 2)")
}

#[test]
fn type_check_ite() -> Result<(), TypeError> {
    tc_test_should_pass("main :: Float\nmain = if false 2.0 3.0")?;
    tc_test_should_pass("main :: Int\nmain = if false 2 3")?;
    tc_test_should_pass("main :: Bool\nmain = if true true false")?;

    tc_test_should_fail("main :: Int\nmain = if false 2.0 3");
    tc_test_should_fail("main :: Float\nmain = if false 2.0 true");
    Ok(())
}

#[test]
fn type_check_const_int_abst() -> Result<(), TypeError> {
    tc_test_should_pass(
        "const_10 :: Float -> Int\nconst_10 = \\x . 10\nmain :: Int\nmain = const_10 2.0",
    )?;

    tc_test_should_fail(
        "const_10 :: Float -> Int\nconst_10 = \\x . 10\nmain :: Int\nmain = const_10 2",
    );

    Ok(())
}

#[test]
fn type_check_abst() -> Result<(), TypeError> {
    tc_test_should_pass("main :: Int\nmain = (\\x y z.x) 10 10 10")?;
    tc_test_should_pass("main :: Int\nmain = (\\x y z a.x) 10 (\\x.x) 10 true")?;
    tc_test_should_pass("main :: Int -> Int\nmain = (\\x.x) (\\x.x)")?;
    tc_test_should_pass("main :: Int\nmain = (\\x.x) ((\\x.x) 10)")?;
    tc_test_should_fail("main :: Int\nmain = (\\x y. y) 10 true");
    tc_test_should_pass("main :: Int\nmain = (\\x y. x) 10 true")?;

    tc_test_should_pass("main :: Int -> Int\nmain = (\\x.x)")?;

    tc_test_should_pass("main :: Int -> Int\nmain = (\\x y. x) (\\x.x) (\\x.x)")
}

#[test]
fn type_y_combinator() {
    tc_test_should_fail(
        "y :: a -> a -> a\ny f = (\\x. f (x x)) (\\x. f (x x))\nmain::Int\nmain = 10",
    );
}

#[test]
fn type_check_extra_arg_should_fail() {
    tc_test_should_fail(
        "const_10 :: Float -> Int\nconst_10 = \\x :: Float. 10\nmain :: Int\nmain = const_10 2.0 10",
    )
}

#[test]
fn type_check_const_abst() -> Result<(), TypeError> {
    tc_test_should_pass("main :: Float\nmain = (\\x y. x) 2.0 20")?;
    tc_test_should_pass("main :: Int\nmain = (\\_ . 10) 2.0")?;
    tc_test_should_fail("main :: Int\nmain = (\\x y . x) 2.0");
    tc_test_should_pass("main :: Int\nmain = (\\x y. y) 2.0 20")?;
    tc_test_should_pass("main :: Int\nmain = (\\x y. y) 2.0 20")?;
    tc_test_should_pass("main :: Int\nmain = (\\x :: Int -> Int. x) (\\x :: Int.x) 20")
}

#[test]
fn type_check_pair() -> Result<(), TypeError> {
    tc_test_should_pass("main :: (a, (b, c)) -> a\nmain (x, (y, z)) = x")?;
    tc_test_should_pass("main :: (a, (b, c)) -> b\nmain  (x, (y, z)) = y")?;
    tc_test_should_pass("main :: (a, (b, c)) -> c\nmain  (x, (y, z)) = z")?;

    tc_test_should_pass("main :: (a, b) -> a\nmain (x, y) = x")?;
    tc_test_should_pass("main :: (a, b) -> b\nmain (x, y) = y")?;

    tc_test_should_pass("main :: a -> b -> (a, b)\nmain x y = (x, y)")?;
    tc_test_should_pass("main :: a -> b -> (a, b)\nmain = \\x y. (x, y)")?;
    tc_test_should_pass("main :: a -> b -> c -> (a, (b, c))\nmain = \\x y z. (x, (y, z))")?;
    tc_test_should_pass(
        "main :: a -> b -> c -> d -> ((a, b), (c, d))\nmain = \\a b c d. ((a, b), (c, d))",
    )
}

fn mod_main_inference_test(program: &str, type_str: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();
    let main_expr_type = ast
        .get(ast.get_main(ast.root).unwrap())
        .clone()
        .type_assignment
        .unwrap();
    assert_eq!(main_expr_type.to_string(), type_str);
}

#[allow(unused)]
fn mod_main_inference_test_no_prelude(program: &str, type_str: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();
    let main_expr_type = ast
        .get(ast.get_main(ast.root).unwrap())
        .clone()
        .type_assignment
        .unwrap();
    assert_eq!(main_expr_type.to_string(), type_str);
}

fn mod_inference_should_fail(program: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm).unwrap_err();
}

fn expr_inference_should_fail(program: &str) {
    let mut pr = Parser::from_string(program.to_string())
        .parse_tl_expression(true)
        .unwrap();
    let expr = pr.ast.root;
    infer_type(&mut pr.ast, expr, &pr.tm).unwrap_err();
}

#[test]
fn stuff() -> Result<(), TypeError> {
    tc_test_should_pass("main:: (a -> b) -> a -> b\nmain f x = f x")?;

    tc_test_should_pass("main :: a -> b -> b\nmain = \\b . if true (\\x . x) (\\x . x)")?;

    expr_inference_should_fail("\\x . x x");

    tc_test_should_pass("main::Int -> Int\nmain = if true (\\x :: Int. x) (\\x . x)")?;
    tc_test_should_pass("main :: Bool -> Int -> Int\nmain = \\b . if b (\\x . x) (\\x . 10)")
}

#[test]
fn alias_test() -> Result<(), TypeError> {
    tc_test_should_pass("type IntAlias = Int\nmain :: IntAlias -> Int\nmain = \\x.x")?;
    tc_test_should_fail("type IntAlias = Bool\nmain :: IntAlias -> Int\nmain = \\x.x");
    tc_test_should_fail("type IntAlias = Int\nmain :: IntAlias -> Bool\nmain = \\x.x");

    Ok(())
}

#[test]
fn maybe_test() -> Result<(), TypeError> {
    tc_test_should_pass(
        "data Maybe2 a = Some2 a | None\nmain :: a -> Maybe2 a\nmain = \\x. Some2 x",
    )?;
    tc_test_should_fail("data Maybe2 a = Some2 a | None\nmain :: a -> Int\nmain = \\x. Some2 x");

    Ok(())
}

#[test]
fn either_test() -> Result<(), TypeError> {
    tc_test_should_fail(
        "data Either2 a b = Left2 a | Right2 b\nmain :: a -> Either2 a b\nmain = \\x. Right2 x",
    );
    tc_test_should_pass(
        "data Either2 a b = Left2 a | Right2 b\nmain :: a -> Either2 b a\nmain = \\x. Left2 x",
    )?;

    Ok(())
}

#[test]
fn list_text() -> Result<(), TypeError> {
    tc_test_should_pass(
        "data List2 a = Cons2 a (List2 a) | Nil2\ndata IntListEither a = List2 (List2 Int) | Right2 a\nmain::Int -> IntListEither a\nmain = \\x.List2 (Cons2 x Nil2)",
    )?;
    tc_test_should_pass(
        "data List2 a = Cons2 a (List2 a) | Nil2\ndata IntListEither a = Left2 (List2 Int) | Right2 a\nmain::Int -> IntListEither a\nmain = \\x.Left2 (Cons2 x (Cons2 10 Nil2))",
    )?;

    tc_test_should_pass("data List2 a = Cons2 a (List2 a) | Nil2\ndata IntListEither a = Left2 (List2 Int) | Right2 a\nmain :: Int -> (IntListEither a)\nmain = \\x.Left2 (Cons2 x (Cons2 10 Nil2))")?;

    Ok(())
}

#[test]
fn triple_test() -> Result<(), TypeError> {
    tc_test_should_pass(
        "data Triple a b c = Triple a b c | NoTriple\nmain :: a -> b -> c -> Triple a b c\nmain = \\x y z. Triple x y z",
    )?;

    tc_test_should_pass(
        "data Triple a b c = Triple a b c | NoTriple\nmain::Triple a b c\nmain = NoTriple",
    )?;

    Ok(())
}

#[test]
fn check_match_is_less_than_2_long() -> Result<(), TypeError> {
    tc_test_should_pass(
        r#"
    len_less_than_2 :: List a -> Bool
    len_less_than_2 lst = match lst {
        | Nil       -> true
        | Cons _ (Cons _ Nil) -> true
        | _ -> false
    }

    main :: Bool
    main = len_less_than_2 (Cons 1 (Cons 2 (Cons 3 Nil)))"#,
    )
}

#[test]
fn check_match_map() -> Result<(), TypeError> {
    tc_test_should_pass(
        r#"
    main :: List Int
    main = map (\x.x) (Cons 1 (Cons 2 (Cons 3 Nil)))"#,
    )
}

#[test]
fn check_match_map_ifnot_zero() -> Result<(), TypeError> {
    let program = r#"
    // Map a function over a list, skipping zeros
    mapNoZero :: (Int -> b) -> List Int -> List b
    mapNoZero f lst = match lst {
        | Nil       -> Nil
        | Cons 0 xs -> mapNoZero f xs
        | Cons x xs -> Cons (f x) (mapNoZero f xs)
    }

    main :: List Int
    main = mapNoZero (\x.x) (Cons 1 (Cons 2 (Cons 3 Nil)))"#;

    mod_main_inference_test(program, "List Int");

    let program = r#"
    // Map a function over a list, skipping zeros
    mapNoZero :: (a -> b) -> List a -> List b
    mapNoZero f lst = match lst {
        | Nil       -> Nil
        | Cons 0 xs -> mapNoZero f xs
        | Cons x xs -> Cons (f x) (mapNoZero f xs)
    }

    main :: List Int
    main = mapNoZero (\x.x) (Cons 1 (Cons 2 (Cons 3 Nil)))"#;

    mod_inference_should_fail(program);

    Ok(())
}

#[test]
fn check_ifmatch() -> Result<(), TypeError> {
    let program = r#"
    main :: Bool -> a -> a -> a
    main cond x y = match cond :: Bool {
    | true -> x 
    | false -> y
    }"#;

    mod_main_inference_test(program, "∀a. Bool -> a -> a -> a");

    Ok(())
}

#[test]
fn bind_io() -> Result<(), TypeError> {
    tc_test_should_pass_no_prelude(
        r#"
    type RealWorld = Int
    data IO a = IO (RealWorld -> (RealWorld, a))

    // Bind over IO
    main :: IO a -> (a -> IO b) -> IO b
    main io f = match io {
        | IO action -> IO (\w. match (action w) {
            | (new_w, x) -> match f x {
                | IO new_action -> new_action new_w // (RealWorld, b)
            } // (RealWorld, b)
        })
    }"#,
    )
}