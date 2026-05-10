use super::*;

impl AST {
    pub fn wait_for_args(&mut self, node: usize) {
        self.vec[node].wait_for_args();
    }

    pub fn fancy_assign_abst_syntax(&mut self, node: usize) {
        self.vec[node].fancy_assign_abst_syntax = true;
    }

    pub fn set_type(&mut self, node: usize, t: Type) {
        self.vec[node].type_assignment = Some(t);
    }

    pub fn add(&mut self, n: ASTNode) -> usize {
        self.vec.push(n);
        self.vec.len() - 1
    }

    pub fn single_node(n: ASTNode) -> Self {
        let mut ast = Self::new();
        let id = ast.add(n);
        ast.root = id;

        ast
    }

    pub fn clone_node(&self, n: usize) -> AST {
        let node = self.get(n);
        let mut ast = AST::single_node(node.clone());
        for i in 0..node.children.len() {
            let index = ast.append_root(&self.clone_node(node.children[i]));
            ast.vec[ast.root].children[i] = index;
        }

        ast
    }

    pub fn add_id(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_id(tk, line, col))
    }

    pub fn add_typed_id(
        &mut self,
        tk: Token,
        line: usize,
        col: usize,
        assigned_type: Type,
    ) -> usize {
        let node = self.add(ASTNode::new_id(tk, line, col));
        self.vec[node].type_assignment = Some(assigned_type);
        node
    }

    pub fn add_lit(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_lit(tk, line, col))
    }

    pub fn add_app(
        &mut self,
        f: usize,
        x: usize,
        line: usize,
        col: usize,
        dollar_app: bool,
    ) -> usize {
        self.add(ASTNode::new_app(f, x, line, col, dollar_app))
    }

    pub fn add_pair(&mut self, a: usize, b: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_pair(a, b, line, col))
    }

    pub fn add_abstraction(&mut self, id: usize, exp: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_abstraction(id, exp, line, col))
    }

    pub fn set_assignment_type(&mut self, assignment: usize, type_: Type) {
        self.vec[assignment].type_assignment = Some(type_);
    }

    pub fn add_assignment(
        &mut self,
        id: usize,
        exp: usize,
        line: usize,
        col: usize,
        t: Option<Type>,
    ) -> usize {
        self.add(ASTNode::new_assignment(id, exp, line, col, t))
    }

    pub fn add_match(&mut self, cases: Vec<usize>, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_match(cases, line, col))
    }

    pub fn add_module(&mut self, assigns: Vec<usize>, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_module(assigns, line, col))
    }

    pub fn add_to_module(&mut self, module: usize, assign: usize) {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);
        self.vec[module].children.push(assign);
    }
}
