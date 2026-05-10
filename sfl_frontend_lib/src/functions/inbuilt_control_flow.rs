use crate::*;

pub fn inbuilt_if(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].get_lit_type(), Type::Primitive(Primitive::Bool));
    if args[0].get_value() == "true" {
        Parser::from_string("\\x y. x".to_string())
            .parse_tl_expression()
            .unwrap()
    } else {
        Parser::from_string("\\x y. y".to_string())
            .parse_tl_expression()
            .unwrap()
    }
}
