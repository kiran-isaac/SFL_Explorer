mod ast;
mod find_redexes;
mod functions;
mod parsing;
mod types;

pub use ast::*;
pub use find_redexes::{
    find_all_redex_contraction_pairs, find_single_redex_contraction_pair, RCPair,
};
pub use functions::KnownTypeLabelTable;
pub use parsing::{Parser, Token};
pub static PRELUDE: &str = include_str!("../../prelude.sfl");
pub use types::{typecheck, typecheck_tl_expr, Primitive, Type, TypeError};

#[cfg(test)]
mod lib_test;

#[cfg(target_arch = "wasm32")]
mod wasm_utils;
#[cfg(target_arch = "wasm32")]
#[allow(unused)]
use wasm_utils::*;
