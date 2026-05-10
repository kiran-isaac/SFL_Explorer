// This is what the AST definition should of looked like
// The overhaul is not worth it. 

struct AST<'a> {
    vec: Vec<ASTNode<'a>>,
    root: &'a ASTNode<'a>,
}

enum ASTNodeType<'a> {
    Identifier{name: String},
    Literal{value: String, _type: PrimitiveType},
    Pair{first: &'a ASTNode<'a>, second: &'a ASTNode<'a>},
    Assignment{to: String, expr: &'a ASTNode<'a>, type_assign: Type},
    Abstraction{var: String, expr: &'a ASTNode<'a>, type_assign: Type},
    Module{assigns: Vec<&'a ASTNode<'a>>},
    Match{expr: &'a ASTNode<'a>, cases: Vec<&'a ASTNode<'a>>}
} 

struct ASTNodeSyntaxInfo { ... }

struct ASTNode<'a> {
    t: ASTNodeType<'a>,
    info: ASTNodeSyntaxInfo
}

// struct ASTNodeSyntaxInfo {...}

// struct ASTNode {
//     t: ASTNodeType,
//     token: Option<Token>,
//     children: Vec<usize>,
//     line: usize,
//     col: usize,
//     type_assignment: Option<Type>,
//     additional_syntax_information: ASTNodeSyntaxInfo
// }