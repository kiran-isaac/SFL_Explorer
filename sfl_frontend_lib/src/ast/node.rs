use crate::parsing::{Token, TokenType};
use crate::{Primitive, Type};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,
    Pair,
    Application,
    Assignment,
    Abstraction,
    Module,
    Match,
}

#[derive(Clone)]
pub struct ASTNode {
    pub t: ASTNodeType,
    pub(super) info: Option<Token>,
    pub(super) children: Vec<usize>,
    pub line: usize,
    pub col: usize,
    pub type_assignment: Option<Type>,
    pub wait_for_args: bool,
    pub fancy_assign_abst_syntax: bool,
    pub dollar_app: bool,
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{:?} ", self.t);
        if let Some(tk) = &self.info {
            s.push_str(&format!("{:?} ", tk.value));
        }

        write!(f, "{}", s)
    }
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Primitive(Primitive::Int64),
                    TokenType::FloatLit => Type::Primitive(Primitive::Float64),
                    TokenType::BoolLit => Type::Primitive(Primitive::Bool),
                    TokenType::StringLit => unimplemented!("String literal type"),
                    _ => panic!("Literal node with bad token"),
                },
                None => panic!("Literal node with no token"),
            },
            _ => panic!("get_lit_type called on non-literal node"),
        }
    }

    /// Get the string value of the identifier or literal
    #[inline(always)]
    pub fn get_value(&self) -> String {
        assert!(self.t == ASTNodeType::Identifier || self.t == ASTNodeType::Literal);
        match &self.info {
            Some(tk) => tk.value.clone(),
            None => panic!("Cannot get value of node {:?}", self),
        }
    }

    pub fn is_uppercase(&self) -> bool {
        if self.t == ASTNodeType::Identifier {
            return self.get_value().chars().nth(0).unwrap().is_uppercase();
        }
        false
    }

    pub(super) fn new_lit(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_id(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_pair(a: usize, b: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Pair,
            info: None,
            children: vec![a, b],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_app(f: usize, x: usize, line: usize, col: usize, dollar: bool) -> Self {
        ASTNode {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, x],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: dollar,
        }
    }

    pub(super) fn new_abstraction(id: usize, exp: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Abstraction,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_assignment(
        id: usize,
        exp: usize,
        line: usize,
        col: usize,
        t: Option<Type>,
    ) -> Self {
        ASTNode {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: t,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_module(assigns: Vec<usize>, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Module,
            info: None,
            children: assigns,
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn new_match(cases: Vec<usize>, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Match,
            info: None,
            children: cases,
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
        }
    }

    pub(super) fn wait_for_args(&mut self) {
        self.wait_for_args = true;
    }
}
