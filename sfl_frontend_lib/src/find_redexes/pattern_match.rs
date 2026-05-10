use crate::{ASTNodeType, AST};
use std::collections::HashMap;

pub enum PatternMatchResult {
    Refute,
    // The bindings if successful
    Success(HashMap<String, usize>),
    Unknown,
}

use PatternMatchResult::*;

fn match_against_app(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);
    #[cfg(debug_assertions)]
    let _pat_str = ast.to_string_sugar(pattern, false);

    assert_eq!(pattern_n.t, ASTNodeType::Application);

    match expr_n.t {
        ASTNodeType::Application => {
            let lhs = pattern_match(ast, ast.get_func(expr), ast.get_func(pattern));
            let rhs = pattern_match(ast, ast.get_arg(expr), ast.get_arg(pattern));
            match (lhs, rhs) {
                (Success(mut lhs), Success(rhs)) => {
                    lhs.extend(rhs);
                    Success(lhs)
                }
                (Unknown, _) | (_, Unknown) => Unknown,
                (Refute, _) | (_, Refute) => Refute,
            }
        }
        ASTNodeType::Pair | ASTNodeType::Literal | ASTNodeType::Abstraction => Refute,
        ASTNodeType::Identifier => {
            if expr_n.is_uppercase() {
                Refute
            } else {
                Unknown
            }
        }
        ASTNodeType::Match => Unknown,
        _ => unreachable!(),
    }
}

fn match_against_pair(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);
    #[cfg(debug_assertions)]
    let _pat_str = ast.to_string_sugar(pattern, false);

    assert_eq!(pattern_n.t, ASTNodeType::Pair);

    match expr_n.t {
        ASTNodeType::Application => {
            // If the head is a constructor then refute
            if ast.get(ast.get_app_head(expr)).is_uppercase() {
                Refute
            } else {
                Unknown
            }
        }
        ASTNodeType::Pair => {
            let lhs = pattern_match(ast, ast.get_first(expr), ast.get_first(pattern));
            let rhs = pattern_match(ast, ast.get_second(expr), ast.get_second(pattern));
            match (lhs, rhs) {
                (Success(mut lhs), Success(rhs)) => {
                    lhs.extend(rhs);
                    Success(lhs)
                }
                (Unknown, _) | (_, Unknown) => Unknown,
                (Refute, _) | (_, Refute) => Refute,
            }
        }
        ASTNodeType::Identifier => {
            if expr_n.is_uppercase() {
                Refute
            } else {
                Unknown
            }
        }
        ASTNodeType::Literal | ASTNodeType::Abstraction => Refute,
        ASTNodeType::Match => Unknown,
        _ => unreachable!(),
    }
}

fn match_against_literal(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);
    #[cfg(debug_assertions)]
    let _pat_str = ast.to_string_sugar(pattern, false);

    assert_eq!(pattern_n.t, ASTNodeType::Literal);

    match expr_n.t {
        ASTNodeType::Application => {
            // If the head is a constructor then refute
            if ast.get(ast.get_app_head(expr)).is_uppercase() {
                Refute
            } else {
                Unknown
            }
        }
        ASTNodeType::Literal => {
            if expr_n.get_value() == pattern_n.get_value() {
                Success(HashMap::new())
            } else {
                Refute
            }
        }
        ASTNodeType::Identifier => {
            if expr_n.is_uppercase() {
                Refute
            } else {
                Unknown
            }
        }
        ASTNodeType::Abstraction | ASTNodeType::Pair => Refute,
        ASTNodeType::Match => Unknown,
        _ => unreachable!(),
    }
}

fn match_against_identifier(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);
    #[cfg(debug_assertions)]
    let _pat_str = ast.to_string_sugar(pattern, false);

    assert_eq!(pattern_n.t, ASTNodeType::Identifier);

    let pat_first_char = pattern_n.get_value().chars().nth(0).unwrap();
    match pat_first_char {
        // Wildcard
        'a'..='z' => {
            let mut map: HashMap<String, usize> = HashMap::new();
            map.insert(pattern_n.get_value(), expr);
            return Success(map);
        }
        // Non binding wildcard
        '_' => return Success(HashMap::new()),

        // Constructor
        'A'..='Z' => match expr_n.t {
            ASTNodeType::Identifier => {
                let expr_first_char = expr_n.get_value().chars().nth(0).unwrap();
                match expr_first_char {
                    'A'..='Z' => {
                        // If its another constructor, then we can refute the pattern
                        if expr_n.get_value() == pattern_n.get_value() {
                            Success(HashMap::new())
                        } else {
                            Refute
                        }
                    }
                    // If its not a constructor we cant resolve the pattern
                    _ => Unknown,
                }
            }
            ASTNodeType::Application => {
                let head_n = ast.get(ast.get_app_head(expr));
                // We can only refute if the head of our application is a constructor as that means
                // it will definitely not eval to this pattern
                if head_n.is_uppercase() {
                    Refute
                } else {
                    Unknown
                }
            }
            ASTNodeType::Literal | ASTNodeType::Pair | ASTNodeType::Abstraction => Refute,
            ASTNodeType::Match => Unknown,
            _ => unreachable!("Not an expression"),
        },
        _ => unreachable!("invalid first char"),
    }
}

/// Get if pattern is matched, and returns bindings
pub fn pattern_match(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    match ast.get(pattern).t {
        ASTNodeType::Identifier => match_against_identifier(ast, expr, pattern),
        ASTNodeType::Application => match_against_app(ast, expr, pattern),
        ASTNodeType::Pair => match_against_pair(ast, expr, pattern),
        ASTNodeType::Literal => match_against_literal(ast, expr, pattern),
        _ => unreachable!(),
    }
}
