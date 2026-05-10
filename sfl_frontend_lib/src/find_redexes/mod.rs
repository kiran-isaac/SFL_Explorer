use crate::{ASTNodeType, AST};
mod reduce;

pub use reduce::{find_all_redex_contraction_pairs, find_single_redex_contraction_pair};

#[derive(Clone)]
pub struct RCPair {
    pub from: usize,
    pub to: AST,
    pub msg_after: String,
    pub msg_before: String,
}

mod pattern_match;
#[cfg(test)]
mod reduce_test;
