use super::AST;
use crate::parsing::{Parser, ParserError};
use crate::ASTDiff;

fn diff_same_as_tostring(str1: &str, str2: &str) -> Result<(), ParserError> {
    let ast1 = Parser::from_string(str1.to_string())
        .parse_module(true)?
        .ast;
    let ast2 = Parser::from_string(str2.to_string())
        .parse_module(true)?
        .ast;

    let ast1_main = ast1.get_assign_exp(ast1.get_main(ast1.root).unwrap());
    let ast2_main = ast2.get_assign_exp(ast2.get_main(ast2.root).unwrap());

    let diff = AST::diff(&ast1, &ast2, ast1_main, ast2_main);
    assert_eq!(diff.str_1(), ast1.to_string_sugar(ast1_main, false));
    assert_eq!(diff.str_2(), ast2.to_string_sugar(ast2_main, false));
    Ok(())
}

fn get_diff(str1: &str, str2: &str) -> Result<ASTDiff, ParserError> {
    let ast1 = Parser::from_string(str1.to_string())
        .parse_module(true)?
        .ast;
    let ast2 = Parser::from_string(str2.to_string())
        .parse_module(true)?
        .ast;

    let ast1_main = ast1.get_assign_exp(ast1.get_main(ast1.root).unwrap());
    let ast2_main = ast2.get_assign_exp(ast2.get_main(ast2.root).unwrap());

    Ok(AST::diff(&ast1, &ast2, ast1_main, ast2_main))
}

#[test]
fn diff_test_5() -> Result<(), ParserError> {
    let diff = get_diff(
        r#"
            square :: Int -> Int
            square x = x * x

            // List of the square numbers from lower to upper
            list_of_squares :: Int -> Int -> List Int
            list_of_squares lower upper = map square $ range lower upper

            main :: Int
            main = sum $ (list_of_squares 1 5)
        "#,
        r#"
            square :: Int -> Int
            square x = x * x

            // List of the square numbers from lower to upper
            list_of_squares :: Int -> Int -> List Int
            list_of_squares lower upper = map square $ range lower upper

            main :: Int
            main = foldr (\x. \acc. x + acc) 0 $ (list_of_squares 1 5)
        "#,
    )?;
    dbg!(diff);
    Ok(())
}

#[test]
fn diff_test_4() -> Result<(), ParserError> {
    let diff = get_diff(
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = 5 * match ((5 - 1) <= 1) {
              | true -> 1
              | false -> (5 - 1) * (fac ((5 - 1) - 1))
            }
        "#,
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = 5 * match (4 <= 1) {
              | true -> 1
              | false -> 4 * (fac (4 - 1))
            }
        "#,
    )?;
    dbg!(diff);
    Ok(())
}

#[test]
fn diff_test_3() -> Result<(), ParserError> {
    diff_same_as_tostring(
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = 5 * (4 * (fac (4 - 1)))
        "#,
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = 5 * (4 * (if ((4 - 1) <= 1) 1 ((4 - 1) * (fac ((4 - 1) - 1)))))
        "#,
    )
}

// #[test]
// fn diff_test1() -> Result<(), ParserError> {
//     diff_same_as_tostring(r#"
//         f :: Int -> Int
//         f n = if ((n % 2) == 0) (n / 2) ((3 * n) + 1)
//
//         // Get collatz sequence
//         collatz :: Int -> List Int
//         collatz n = (\x. if (n <= 1) (Nil) (Cons x (collatz x))) $ f n
//
//         main :: List Int
//         main = collatz 12
//     "#, r#"
//         f :: Int -> Int
//         f n = if ((n % 2) == 0) (n / 2) ((3 * n) + 1)
//
//         // Get collatz sequence
//         collatz :: Int -> List Int
//         collatz n = (\x. if (n <= 1) (Nil) (Cons x (collatz x))) $ f n
//
//         main :: List Int
//         main = (\x. if (12 <= 1) Nil (Cons x (collatz x))) $ f 12
//     "#)
// }

#[test]
fn diff_test_fac() -> Result<(), ParserError> {
    diff_same_as_tostring(
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = fac 5
        "#,
        r#"
            fac :: Int -> Int
            fac n = if (n <= 1) 1 (n * (fac (n - 1)))

            main :: Int
            main = if (5 <= 1) 1 (5 * (fac (5 - 1)))
        "#,
    )
}
