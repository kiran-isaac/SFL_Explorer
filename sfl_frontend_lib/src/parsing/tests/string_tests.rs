use crate::parsing::*;

#[test]
fn assign() -> Result<(), ParserError> {
    let str = "x = 'a'";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module(false)?.ast;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_assign_exp(assign);

    let node = ast.get(exp);

    println!("{}", ast.to_string_sugar(exp, true));

    Ok(())
}