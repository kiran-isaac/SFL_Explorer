use parsing::{Token, TokenType};

use super::assert_prim_type;
use crate::*;
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

fn inbuilt_binary<T>(call: &ASTNode, args: Vec<&ASTNode>, op: fn(T, T) -> T, p: Primitive) -> AST
where
    T: FromStr + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Display,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let a = args[0];
    let b = args[1];

    assert_prim_type(&a.get_lit_type(), p);
    assert_prim_type(&b.get_lit_type(), p);

    let a_int: T = a.get_value().parse().unwrap();
    let b_int: T = b.get_value().parse().unwrap();

    let c_int = op(b_int, a_int);
    let mut ast = AST::new();
    ast.add_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", c_int),
        },
        call.line,
        call.col,
    );
    ast
}

pub fn inbuilt_int_add(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, i64::add, Primitive::Int64)
}

pub fn inbuilt_int_sub(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, i64::sub, Primitive::Int64)
}

pub fn inbuilt_int_mul(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, i64::mul, Primitive::Int64)
}

pub fn inbuilt_int_div(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, i64::div, Primitive::Int64)
}

pub fn inbuilt_int_mod(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(
        call,
        args,
        |x: i64, y: i64| x - ((x / y) * y),
        Primitive::Int64,
    )
}

pub fn inbuilt_int_neg(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    let x: i64 = args[0].get_value().parse().unwrap();
    let mut ast = AST::new();
    ast.add_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", -x),
        },
        call.line,
        call.col,
    );
    ast
}

pub fn inbuilt_float_add(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, f64::add, Primitive::Float64)
}

pub fn inbuilt_float_sub(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, f64::sub, Primitive::Float64)
}

pub fn inbuilt_float_mul(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, f64::mul, Primitive::Float64)
}

pub fn inbuilt_float_div(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary(call, args, f64::div, Primitive::Float64)
}

pub fn inbuilt_float_neg(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    let x: f64 = args[0].get_value().parse().unwrap();
    let mut ast = AST::new();
    ast.add_lit(
        Token {
            tt: TokenType::FloatLit,
            value: format!("{}", -x),
        },
        call.line,
        call.col,
    );
    ast
}

#[cfg(test)]
pub fn inbuilt_int_zero(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 0);

    let mut ast = AST::new();
    ast.add_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", 0),
        },
        call.line,
        call.col,
    );
    ast
}

fn inbuilt_binary_boolean<T>(
    call: &ASTNode,
    args: Vec<&ASTNode>,
    op: fn(T, T) -> bool,
    p: Primitive,
) -> AST
where
    T: FromStr + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Display,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let a = args[0];
    let b = args[1];

    assert_prim_type(&a.get_lit_type(), p);
    assert_prim_type(&b.get_lit_type(), p);

    let a_int: T = a.get_value().parse().unwrap();
    let b_int: T = b.get_value().parse().unwrap();

    let c_int = op(b_int, a_int);

    let mut ast = AST::new();
    ast.add_lit(
        Token {
            tt: TokenType::BoolLit,
            value: format!("{}", if c_int { "true" } else { "false" }),
        },
        call.line,
        call.col,
    );
    ast
}

pub fn inbuilt_int_eq(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: i64, y: i64| x == y, Primitive::Int64)
}

pub fn inbuilt_int_lt(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: i64, y: i64| x < y, Primitive::Int64)
}

pub fn inbuilt_int_gt(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: i64, y: i64| x > y, Primitive::Int64)
}

pub fn inbuilt_int_lte(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: i64, y: i64| x <= y, Primitive::Int64)
}

pub fn inbuilt_int_gte(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: i64, y: i64| x >= y, Primitive::Int64)
}

pub fn inbuilt_float_eq(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: f64, y: f64| x == y, Primitive::Float64)
}

pub fn inbuilt_float_lt(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: f64, y: f64| x < y, Primitive::Float64)
}

pub fn inbuilt_float_gt(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: f64, y: f64| x > y, Primitive::Float64)
}

pub fn inbuilt_float_lte(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: f64, y: f64| x <= y, Primitive::Float64)
}

pub fn inbuilt_float_gte(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    inbuilt_binary_boolean(call, args, |x: f64, y: f64| x >= y, Primitive::Float64)
}
