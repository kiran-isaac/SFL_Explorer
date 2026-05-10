use crate::parsing::*;

#[test]
fn assign() -> Result<(), ParserError> {
    let str = "x = add 2 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_assign_exp(assign);

    let left = ast.get_func(exp);
    let right = ast.get_arg(exp);
    assert!(ast.get(right).get_value() == "5");
    assert!(ast.get(ast.get_func(left)).get_value() == "add");
    assert!(ast.get(ast.get_arg(left)).get_value() == "2");

    Ok(())
}

#[test]
fn dollar() -> Result<(), ParserError> {
    let str = "y = (\\x.x) $ 5 + 1 \n x = 10";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let string = ast.to_string_sugar(0, false);
    println!("{}", string);

    Ok(())
}

#[test]
fn assign_2() -> Result<(), ParserError> {
    let str = "x = add (y z) 5";
    let mut parser = Parser::from_string(str.to_string());

    parser.bind("y".to_string());
    parser.bind("z".to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_assign_exp(assign);

    let left = ast.get_func(exp);
    let right = ast.get_arg(exp);
    assert!(ast.get(right).get_value() == "5");
    assert!(ast.get(ast.get_func(left)).get_value() == "add");

    let y_z = ast.get_arg(left);
    assert!(ast.get(ast.get_func(y_z)).get_value() == "y");
    assert!(ast.get(ast.get_arg(y_z)).get_value() == "z");

    println!("{}", ast.to_string_sugar(module, false));

    Ok(())
}

#[test]
fn multi_assign() -> Result<(), ParserError> {
    let str = "x = 5\n\n//Hello\ny = 6\nz = 7";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;

    let ass1 = ast.get_assign_to(module, "x".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass1)).get_value() == "5");

    let ass2 = ast.get_assign_to(module, "y".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass2)).get_value() == "6");

    let ass3 = ast.get_assign_to(module, "z".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass3)).get_value() == "7");

    assert!(ast.to_string_sugar(module, false) == "x = 5\ny = 6\nz = 7".to_string());

    Ok(())
}

#[test]
fn bound() -> Result<(), ParserError> {
    // recursive
    let str = "x = x 5";
    Parser::from_string(str.to_string())
        .parse_module(false)?
        .ast;

    // add is an inbuilt
    let str = "x = add 2 x";
    Parser::from_string(str.to_string())
        .parse_module(false)?
        .ast;

    // y is unbound
    let str = "x = add 2 y";
    assert!(Parser::from_string(str.to_string())
        .parse_module(false)
        .is_err());

    Ok(())
}

fn unchanged_parse_output_str_test(program_str: &str, types: bool) -> Result<(), ParserError> {
    let mut parser = Parser::from_string(program_str.to_string());
    let ast = parser.parse_module(false)?.ast;
    assert_eq!(program_str, ast.to_string_sugar(ast.root, types));
    Ok(())
}

#[test]
fn infix_expr() -> Result<(), ParserError> {
    unchanged_parse_output_str_test("x = 1 + 1", false)?;
    unchanged_parse_output_str_test("x = 1 - 1", false)?;
    unchanged_parse_output_str_test("x = 1 * 1", false)?;
    unchanged_parse_output_str_test("x = 1 / 1", false)?;
    unchanged_parse_output_str_test("x = 1 == 1", false)?;
    unchanged_parse_output_str_test("x = 1 > 1", false)?;
    unchanged_parse_output_str_test("x = 1 < 1", false)?;
    unchanged_parse_output_str_test("x = 1 >= 1", false)?;
    unchanged_parse_output_str_test("x = 1 <= 1", false)?;

    Ok(())
}

#[test]
fn fancy_abst_syntax_test() -> Result<(), ParserError> {
    let program = "inc x = x + 1";
    unchanged_parse_output_str_test(program, false)?;
    Ok(())
}

#[test]
fn abstraction() -> Result<(), ParserError> {
    let str = "x = \\y :: Int. add y 5";
    let mut parser = Parser::from_string(str.to_string());

    let _ = parser.parse_module(false)?.ast;

    // Should error because y is not bound
    let unbound_str = "x = (\\y . add y 5) y";
    let mut parser = Parser::from_string(unbound_str.to_string());
    assert!(parser.parse_module(false).is_err());

    // Should be same for both
    let multi_abstr = "x = \\y :: Int z :: Int . add y 5";
    let multi_abstr2 = "x = \\y :: Int . \\z :: Int . add y 5";
    let ast = Parser::from_string(multi_abstr.to_string())
        .parse_module(false)?
        .ast;
    let ast2 = Parser::from_string(multi_abstr2.to_string())
        .parse_module(false)?
        .ast;
    assert_eq!(
        ast.to_string_sugar(ast.root, false),
        ast2.to_string_sugar(ast2.root, false)
    );

    let ignore_directive = "x = \\_ :: Int . 1.5";
    Parser::from_string(ignore_directive.to_string())
        .parse_module(false)?
        .ast;

    Ok(())
}

#[test]
fn type_assignment() -> Result<(), ParserError> {
    let str = "x :: Int\nx = 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();

    let type_assignment = ast.get(assign).type_assignment.clone();
    assert!(type_assignment.is_some());
    assert_eq!(type_assignment.unwrap().to_string(), "Int".to_string());

    let str = "id2 :: var -> var\nid2 = \\x.x";
    let mut parser = Parser::from_string(str.to_string());
    let ast = parser.parse_module(false)?.ast;
    let module = ast.root;
    let assign = ast.get_assign_to(module, "id2".to_string()).unwrap();
    let type_assignment = ast.get(assign).type_assignment.clone();
    assert!(type_assignment.is_some());
    assert_eq!(
        type_assignment.unwrap().to_string(),
        "∀var. var -> var".to_string()
    );

    Ok(())
}

#[test]
fn type_assignment_right_assoc() -> Result<(), ParserError> {
    let str = "x :: (Int -> Int) -> (Int -> Float) -> Int\nx = 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();

    let type_assignment = ast.get(assign).type_assignment.clone();
    assert!(type_assignment.is_some());
    assert_eq!(
        format!("{:?}", type_assignment.unwrap()),
        "(Int -> Int) -> ((Int -> Float) -> Int)"
    );

    Ok(())
}

#[test]
fn pair() -> Result<(), ParserError> {
    unchanged_parse_output_str_test("pair x y = (x, y)", false)?;
    unchanged_parse_output_str_test("fst (x, y) = x", false)?;
    unchanged_parse_output_str_test("snd (x, y) = y", false)?;
    unchanged_parse_output_str_test("third (x, (y, z)) = z", false)?;

    // unchanged_parse_output_str_test("pair :: a -> b -> (a, b)\npair x y = (x, y)", true)?;
    // unchanged_parse_output_str_test("fst :: (a, b) -> a\nfst (x, y) = x", true)?;
    // unchanged_parse_output_str_test("snd :: (a, b) -> b\nsnd (x, y) = y", true)?;
    // unchanged_parse_output_str_test("third :: (a, (b, c)) -> c\nthird (_, (_, z)) = z", true)?;

    let str = "pair :: a -> b -> (a, b)\npair x y = (x, y)";
    let mut parser = Parser::from_string(str.to_string());
    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    assert_eq!(
        ast.to_string_sugar(module, true),
        "pair :: ∀a. ∀b. a -> b -> (a, b)\npair x y = (x, y)"
    );
    Ok(())
}

#[test]
fn type_decl() -> Result<(), ParserError> {
    let str = "type Bingus = Int\nmain :: Bingus -> Int\nmain = \\x.x";
    let ast = Parser::from_string(str.to_string())
        .parse_module(false)?
        .ast;
    assert_eq!(
        format!("{}", ast.to_string_sugar(ast.root, true)),
        "main :: Bingus -> Int\nmain = \\x. x"
    );

    Ok(())
}

#[test]
fn data_decl() -> Result<(), ParserError> {
    let str = "data Maybe a = Some a | None\nmain :: Int -> Maybe Int\nmain = Some 10";
    let pr = Parser::from_string(str.to_string()).parse_module(false)?;
    let lt = pr.lt;
    let tm = pr.tm;

    assert_eq!(
        format!("{}", lt.get_type("Some").unwrap().unwrap()),
        "∀a. a -> Maybe a"
    );
    assert_eq!(
        format!("{}", lt.get_type("None").unwrap().unwrap()),
        "∀a. Maybe a"
    );
    assert_eq!(
        format!("{}", tm.types.get("Maybe").unwrap().to_string()),
        "∀a. Maybe a"
    );

    Ok(())
}

#[test]
fn data_decl2() -> Result<(), ParserError> {
    let str = "data Maybe a = Some a | None\ndata DataTest maybevar = Bingus (Maybe maybevar)\nmain :: Int -> Maybe Int\nmain = Some 10";
    let pr = Parser::from_string(str.to_string()).parse_module(false)?;
    let lt = pr.lt;
    let tm = pr.tm;

    assert_eq!(
        format!("{}", lt.get_type("Some").unwrap().unwrap()),
        "∀a. a -> Maybe a"
    );
    assert_eq!(
        format!("{}", lt.get_type("None").unwrap().unwrap()),
        "∀a. Maybe a"
    );
    assert_eq!(
        format!("{}", lt.get_type("Bingus").unwrap().unwrap()),
        "∀maybevar. Maybe maybevar -> DataTest maybevar"
    );
    assert_eq!(
        format!("{}", tm.types.get("Maybe").unwrap().to_string()),
        "∀a. Maybe a"
    );
    assert_eq!(
        format!("{}", tm.types.get("DataTest").unwrap().to_string()),
        "∀maybevar. DataTest maybevar"
    );

    Ok(())
}

#[test]
fn list_decl() -> Result<(), ParserError> {
    let str = "data List a = Cons a (List a) | Nil\ndata IntListEither a = Left (List Int) | Right a\nmain :: Int -> (IntListEither a)\nmain = \\x.Left (Cons x Nil)";
    let pr = Parser::from_string(str.to_string()).parse_module(false)?;
    let lt = pr.lt;
    let tm = pr.tm;

    assert_eq!(
        format!("{}", lt.get_type("Cons").unwrap().unwrap()),
        "∀a. a -> List a -> List a"
    );
    assert_eq!(
        format!("{}", lt.get_type("Nil").unwrap().unwrap()),
        "∀a. List a"
    );
    assert_eq!(
        format!("{}", tm.types.get("List").unwrap().to_string()),
        "∀a. List a"
    );
    assert_eq!(
        format!("{}", tm.types.get("IntListEither").unwrap().to_string()),
        "∀a. IntListEither a"
    );
    assert_eq!(
        format!("{}", lt.get_type("Left").unwrap().unwrap()),
        "∀a. List Int -> IntListEither a"
    );
    assert_eq!(
        format!("{}", lt.get_type("Right").unwrap().unwrap()),
        "∀a. a -> IntListEither a"
    );

    Ok(())
}

#[test]
fn list_maybe() -> Result<(), ParserError> {
    let program = r#"
    fac :: Int -> Int
    fac n = if (n <= 1) (1) (n * (fac (n - 1)))

    fromMaybes :: Either a (Maybe a) -> List a
    fromMaybes a = a

    main = Just (fac 5)"#;

    let pr = Parser::from_string(program.to_string()).parse_module(true)?;
    // println!("{:?}", pr.lt);
    println!("{:?}", pr.lt.get_type("fromMaybes").unwrap());

    Ok(())
}

fn assert_string_equal_no_whitespace_or_brackets(s1: String, s2: String) {
    let mut s1 = s1.clone();
    let mut s2 = s2.clone();
    s1.retain(|c| !(c.is_whitespace() || c == ')' || c == '('));
    s2.retain(|c| !(c.is_whitespace() || c == ')' || c == '('));

    assert_eq!(s1, s2);
}

#[test]
fn parse_match_length() -> Result<(), ParserError> {
    let program = r#"
    data List a = Cons a (List a) | Nil

    length x = match x :: List a {
       | Nil       -> 0
       | Cons _ xs -> 1 + length xs
    }

    main = length (Cons 1 (Cons 2 (Cons 3 Nil)))"#;

    let pr = Parser::from_string(program.to_string()).parse_module(false)?;
    assert_string_equal_no_whitespace_or_brackets(
        pr.ast.to_string_sugar(pr.ast.root, false),
        r#"
    length x = match (x :: List a) {
       | Nil       -> 0
       | Cons _ xs -> 1 + (length xs)
    }
    main = length (Cons 1 (Cons 2 (Cons 3 Nil)))"#
            .to_string(),
    );

    Ok(())
}
#[test]
fn parse_match_map() -> Result<(), ParserError> {
    let program = r#"
    data List a = Cons a (List a) | Nil

    map f lst = match lst:: List a {
       | Nil       -> Nil
       | Cons x xs -> Cons (f x) (map f xs)
    }

    main = map (\x. x + 1) (Cons 1 (Cons 2 (Cons 3 Nil)))"#;

    let pr = Parser::from_string(program.to_string()).parse_module(false)?;
    assert_string_equal_no_whitespace_or_brackets(
        pr.ast.to_string_sugar(pr.ast.root, false),
        r#"
    map f lst = match lst :: List a  {
       | Nil       -> Nil
       | Cons x xs -> Cons (f x) (map f xs)
    }
    main = map (\x. x + 1)  (Cons 1 (Cons 2 (Cons 3 Nil)))"#
            .to_string(),
    );

    Ok(())
}

#[test]
fn parse_match_fold() -> Result<(), ParserError> {
    let program = r#"
    data List a = Cons a (List a) | Nil

    foldr f acc lst = match lst :: List a {
       | Nil       -> acc
       | Cons x xs -> f acc (foldr f acc xs)
    }
    
    foldl f acc lst = match lst :: List a {
       | Nil       -> acc
       | Cons x xs -> foldl f (f acc x) xs
    }

    foldr_sum = foldr (\x y. x + y) (Cons 1 (Cons 2 (Cons 3 Nil)))
    foldl_sum = foldl (\x y. x + y) (Cons 1 (Cons 2 (Cons 3 Nil)))
    "#;

    let pr = Parser::from_string(program.to_string()).parse_module(false)?;
    assert_string_equal_no_whitespace_or_brackets(
        pr.ast.to_string_sugar(pr.ast.root, false),
        r#"
    foldr f acc lst = match lst :: List a {
       | Nil       -> acc
       | Cons x xs -> f acc (foldr f acc xs)
    }
    
    foldl f acc lst = match lst :: List a {
       | Nil       -> acc
       | Cons x xs -> foldl f (f acc x) xs
    }

    foldr_sum = foldr (\x . \y. x + y) (Cons 1 (Cons 2 (Cons 3 Nil)))
    foldl_sum = foldl (\x . \y. x + y) (Cons 1 (Cons 2 (Cons 3 Nil)))
    "#
        .to_string(),
    );

    Ok(())
}
