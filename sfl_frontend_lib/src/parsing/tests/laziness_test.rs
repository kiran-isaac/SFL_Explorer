use crate::{find_all_redex_contraction_pairs, Parser};

#[test]
fn test_laziness() {
    let program = "(\\x y. x) ((\\x. 1) true) ((\\x. x + 1) 2)".to_string();
    let pr = Parser::from_string(program)
        .parse_tl_expression(false)
        .unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let rcs = find_all_redex_contraction_pairs(&ast, None, ast.root, &lt);
    let rc = ast.get_laziest_rc(ast.root, &rcs).unwrap();
    let s2 = rc.to.to_string_sugar(rc.to.root, false);
    assert_eq!(s2, "(\\x. 1) true");
}
