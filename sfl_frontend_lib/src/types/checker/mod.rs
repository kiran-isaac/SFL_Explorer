mod checking;
mod context;
mod subtype;
mod synthesis;

use super::Type;
use crate::parsing::TypeMap;
use crate::{KnownTypeLabelTable, AST};
use checking::check_type;
use context::*;
use std::collections::HashSet;
use subtype::subtype;
use synthesis::synthesize_type;

#[derive(Clone, PartialEq, Eq)]
pub struct TypeError {
    pub e: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Debug for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Type Error at [{}:{}]: {}",
            self.line + 1,
            self.col + 1,
            self.e
        )
    }
}
fn type_error(msg: String, ast: &AST, expr: usize) -> TypeError {
    let n = ast.get(expr);
    TypeError {
        e: msg,
        col: n.col,
        line: n.line,
    }
}

static MUST_ASSIGN: bool = true;

pub fn typecheck_tl_expr(expected: &Type, ast: &AST, expr: usize) -> Result<(), TypeError> {
    match check_type(
        Context::from_labels(&KnownTypeLabelTable::new(), &HashSet::new()),
        expected,
        ast,
        expr,
        &TypeMap::new(),
        false,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn infer_type_with_context(
    c: Context,
    ast: &AST,
    expr: usize,
    type_map: &TypeMap,
) -> Result<(Type, Context), TypeError> {
    let (t, c) = synthesize_type(c, ast, expr, type_map, false)?;

    let t = c.substitute(&t);
    Ok((t, c))
}

#[cfg(test)]
pub fn infer_type(ast: &AST, expr: usize, type_map: &TypeMap) -> Result<Type, TypeError> {
    let lt = KnownTypeLabelTable::new();
    let c = Context::from_labels(&lt, &HashSet::new());

    Ok(infer_type_with_context(c, ast, expr, type_map)?
        .0
        .forall_ify())
}

pub fn typecheck(
    ast: &mut AST,
    module: usize,
    lt: &mut KnownTypeLabelTable,
    type_map: &TypeMap,
) -> Result<(), TypeError> {
    let mut c = Context::from_labels(
        &lt,
        &ast.get_assignee_names(module)
            .iter()
            .map(|s| s.clone())
            .collect(),
    );

    for assign_var in &ast.get_assignee_names(module) {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &c);

        let assign = ast.get_assign_to(module, assign_var.clone()).unwrap();

        #[cfg(debug_assertions)]
        let _assign_str = format!("{}", ast.to_string_sugar(assign, false));

        let assign_expr = ast.get_assign_exp(assign);

        match &ast.get(assign).type_assignment {
            Some(type_assignment) => {
                c = c.append(ContextItem::TypeAssignment(
                    assign_var.clone(),
                    Ok(type_assignment.clone()),
                ));
                c = check_type(c, &type_assignment, ast, assign_expr, type_map, false)?
                    .assigns_only();
            }
            None => {
                if MUST_ASSIGN {
                    return Err(type_error(
                        format!("Cannot find type assignment for:  {}", &assign_var),
                        ast,
                        assign_expr,
                    ));
                }

                let c2 = c.append(ContextItem::TypeAssignment(
                    assign_var.clone(),
                    Err(type_error(format!("Cannot infer type of expression containing recursive call. Assign a type to label '{}'", &assign_var), ast, assign_expr)),
                ));
                let (t, _) = infer_type_with_context(c2, &ast, assign_expr, type_map)?;

                let t = t.forall_ify();
                c = c.assigns_only().append(ContextItem::TypeAssignment(
                    assign_var.clone(),
                    Ok(t.clone()),
                ));
                ast.set_assignment_type(assign, t);
            }
        };
    }

    Ok(())
}
