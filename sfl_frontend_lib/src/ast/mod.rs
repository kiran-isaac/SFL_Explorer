mod building;
mod node;
mod output;
mod transform;

#[cfg(test)]
mod diff_tests;

use crate::{find_redexes::RCPair, Token, Type};
pub use node::*;
pub use output::{ASTDiff, ASTDiffElem};
use std::collections::HashSet;
use std::iter::zip;
use std::{collections::HashMap, fmt::Debug, vec};

#[derive(Clone)]
pub struct AST {
    vec: Vec<ASTNode>,
    pub root: usize,
}

impl AST {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            root: 0,
        }
    }

    pub fn rc_to_str(&self, rc: &RCPair) -> String {
        self.to_string_sugar(rc.from, false) + " -> " + &rc.to.to_string_sugar(rc.to.root, false)
    }

    pub fn print_vec_string(&self) -> String {
        let mut s = String::new();
        for n in &self.vec {
            s.push_str(&format!("{:?}\n", n));
        }
        s
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> &ASTNode {
        &self.vec[i]
    }

    pub fn get_first(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[0]
    }

    pub fn get_second(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[1]
    }

    pub fn get_abstr_var(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
        self.vec[abst].children[0]
    }

    pub fn get_abstr_expr(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
        self.vec[abst].children[1]
    }

    pub fn get_func(&self, app: usize) -> usize {
        assert_eq!(self.vec[app].t, ASTNodeType::Application);
        self.vec[app].children[0]
    }

    pub fn get_arg(&self, app: usize) -> usize {
        assert_eq!(self.vec[app].t, ASTNodeType::Application);
        self.vec[app].children[1]
    }

    pub fn get_assign_exp(&self, assign: usize) -> usize {
        assert_eq!(self.vec[assign].t, ASTNodeType::Assignment);
        self.vec[assign].children[1]
    }

    pub fn get_assignee(&self, assign: usize) -> String {
        assert_eq!(self.vec[assign].t, ASTNodeType::Assignment);
        self.get(self.vec[assign].children[0]).get_value().clone()
    }

    pub fn get_assignee_names(&self, module: usize) -> Vec<String> {
        let mut names = Vec::new();
        let assigns = &self.vec[module].children;
        names.reserve_exact(assigns.len());
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            names.push(id.get_value());
        }

        names
    }

    /// Get the main assignment (not just the expr)
    pub fn get_main(&self, module: usize) -> Option<usize> {
        self.get_assign_to(module, "main".to_string())
    }

    // Get assignment within module
    pub fn get_assign_to(&self, module: usize, name: String) -> Option<usize> {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);

        let assigns = &self.vec[module].children;
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            if id.get_value() == name {
                return Some(*a);
            }
        }

        None
    }

    pub fn get_assigns_map(&self, module: usize) -> HashMap<String, usize> {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);
        let mut assigns = HashMap::new();

        for a in &self.vec[module].children {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            assigns.insert(id.get_value(), *a);
        }

        assigns
    }

    pub fn get_match_unpack_pattern(&self, match_: usize) -> usize {
        assert_eq!(self.vec[match_].t, ASTNodeType::Match);
        assert!(self.vec[match_].children.len() > 1);
        self.vec[match_].children[0]
    }

    /// returns patterns to expressions
    pub fn get_match_cases(&self, match_: usize) -> Vec<(usize, usize)> {
        assert_eq!(self.vec[match_].t, ASTNodeType::Match);
        let new_vec = self.vec[match_].children.clone()[1..].to_vec();
        match new_vec.len() % 2 {
            0 => {
                let mut cases = vec![];
                for i in 0..new_vec.len() / 2 {
                    cases.push((new_vec[i * 2], new_vec[i * 2 + 1]));
                }
                cases
            }
            _ => panic!("Match cases must be in pairs"),
        }
    }

    pub fn expr_eq(&self, expr1: usize, expr2: usize) -> bool {
        AST::eq(&self, &self, expr1, expr2)
    }

    pub fn eq(ast1: &AST, ast2: &AST, expr1: usize, expr2: usize) -> bool {
        let n1 = ast1.get(expr1);
        let n2 = ast2.get(expr2);

        match (n1.t, n2.t) {
            (ASTNodeType::Identifier, ASTNodeType::Identifier)
            | (ASTNodeType::Literal, ASTNodeType::Literal) => n1.get_value() == n2.get_value(),
            (a, b) => {
                if a != b {
                    return false;
                }

                for (c1, c2) in zip(&n1.children, &n2.children) {
                    if !AST::eq(ast1, ast2, *c1, *c2) {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn get_app_head(&self, expr: usize) -> usize {
        match self.get(expr).t {
            ASTNodeType::Application => self.get_app_head(self.get_func(expr)),
            _ => expr,
        }
    }
}

impl Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_sugar(self.root, false))
    }
}
