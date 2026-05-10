use parsing::Token;

use super::super::*;
use crate::AST;

#[test]
fn test_basic_int_arith() {
    let mut ast = AST::new();
    let labels = KnownTypeLabelTable::new();
    let add = labels.get(&"add".to_string()).unwrap();
    let sub = labels.get(&"sub".to_string()).unwrap();
    let mul = labels.get(&"mul".to_string()).unwrap();
    let div = labels.get(&"div".to_string()).unwrap();

    for _ in 0..1000 {
        // Generate random numbers, (16 bit to avoid overflow)
        let a_int = rand::random::<i16>();
        let b_int = {
            let mut rand = rand::random::<i16>();
            while rand == 0 {
                rand = rand::random::<i16>();
            }
            rand
        };

        let a = Token {
            tt: parsing::TokenType::IntLit,
            value: format!("{}", a_int),
        };
        let b = Token {
            tt: parsing::TokenType::IntLit,
            value: format!("{}", b_int),
        };

        let a_int = a_int as i64;
        let b_int = b_int as i64;

        let a = ast.add_lit(a, 0, 0);
        let b = ast.add_lit(b, 0, 0);

        let mut call_ast = AST::new();
        call_ast.add_id(
            Token {
                tt: parsing::TokenType::Id,
                value: "_".to_string(),
            },
            0,
            0,
        );
        let call = call_ast.get(0);

        let c_add = add.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_sub = sub.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_mul = mul.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_div = div.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);

        matches!(
            c_add.get(0).get_lit_type(),
            Type::Primitive(Primitive::Int64)
        );
        matches!(
            c_sub.get(0).get_lit_type(),
            Type::Primitive(Primitive::Int64)
        );
        matches!(
            c_mul.get(0).get_lit_type(),
            Type::Primitive(Primitive::Int64)
        );
        matches!(
            c_div.get(0).get_lit_type(),
            Type::Primitive(Primitive::Int64)
        );

        assert_eq!(
            c_add.get(0).get_value().parse::<i64>().unwrap(),
            a_int + b_int
        );
        assert_eq!(
            c_sub.get(0).get_value().parse::<i64>().unwrap(),
            a_int - b_int
        );
        assert_eq!(
            c_mul.get(0).get_value().parse::<i64>().unwrap(),
            a_int * b_int
        );
        assert_eq!(
            c_div.get(0).get_value().parse::<i64>().unwrap(),
            a_int / b_int
        );
    }
}

#[test]
fn test_basic_float_arith() {
    let mut ast = AST::new();
    let labels = KnownTypeLabelTable::new();
    let add = labels.get(&"addf".to_string()).unwrap();
    let sub = labels.get(&"subf".to_string()).unwrap();
    let mul = labels.get(&"mulf".to_string()).unwrap();
    let div = labels.get(&"divf".to_string()).unwrap();

    for _ in 0..1000 {
        // Generate random numbers, (16 bit to avoid overflow)
        let a_float = rand::random::<f64>() * 10.;
        let b_float = rand::random::<f64>() * 10.;

        let a = Token {
            tt: parsing::TokenType::FloatLit,
            value: format!("{}", a_float),
        };
        let b = Token {
            tt: parsing::TokenType::FloatLit,
            value: format!("{}", b_float),
        };

        let a = ast.add_lit(a, 0, 0);
        let b = ast.add_lit(b, 0, 0);

        let mut call_ast = AST::new();
        call_ast.add_id(
            Token {
                tt: parsing::TokenType::Id,
                value: "_".to_string(),
            },
            0,
            0,
        );
        let call = call_ast.get(0);

        let c_add = add.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_sub = sub.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_mul = mul.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);
        let c_div = div.call_inbuilt(&call, vec![ast.get(b), ast.get(a)]);

        matches!(
            c_add.get(0).get_lit_type(),
            Type::Primitive(Primitive::Float64)
        );
        matches!(
            c_sub.get(0).get_lit_type(),
            Type::Primitive(Primitive::Float64)
        );
        matches!(
            c_mul.get(0).get_lit_type(),
            Type::Primitive(Primitive::Float64)
        );
        matches!(
            c_div.get(0).get_lit_type(),
            Type::Primitive(Primitive::Float64)
        );

        assert_eq!(
            c_add.get(0).get_value().parse::<f64>().unwrap(),
            a_float + b_float
        );
        assert_eq!(
            c_sub.get(0).get_value().parse::<f64>().unwrap(),
            a_float - b_float
        );
        assert_eq!(
            c_mul.get(0).get_value().parse::<f64>().unwrap(),
            a_float * b_float
        );
        assert_eq!(
            c_div.get(0).get_value().parse::<f64>().unwrap(),
            a_float / b_float
        );
    }
}
