use super::pattern_match::PatternMatchResult;
use super::*;
use crate::find_redexes::pattern_match::pattern_match;
use crate::functions::KnownTypeLabelTable;
use std::collections::HashMap;

fn comma_ify(vec: Vec<String>) -> String {
    match vec.len() {
        0 => String::new(),
        1 => vec[0].clone(),
        _ => {
            let mut str = vec[0].clone();
            let last = vec[vec.len() - 1].clone();
            for string in &vec[1..vec.len() - 1] {
                str += ", ";
                str += string;
            }
            str + " and " + &last
        }
    }
}

/// This will check for applications to functions:
/// - lables with func types
/// - lambda abstractions
/// - inbuilt functions
/// with the right num of args.
/// For example, a call to a inbuilt add could be: add 2 3
/// Which would look like
/// App[[App add 2], 3]
/// This function checks that the rhs is a literal, and the lhs is
/// either a function or an App of a function in the set of funcs
/// with the right num of args
fn check_for_valid_call(
    ast: &AST,
    expr: usize,
    lt: &KnownTypeLabelTable,
    am: HashMap<String, usize>,
) -> Option<RCPair> {
    let mut f = ast.get_func(expr);
    let mut x = ast.get_arg(expr);
    let mut argv = vec![];
    let mut argv_ids = vec![];

    let mut argv_strs = vec![];

    // True if only literals encountered. If true, then we can call inbuilt functions
    let mut literals_only = true;

    loop {
        argv.push(ast.get(x));
        argv_ids.push(x);

        let arg_str = ast.to_string_sugar(x, false);
        match ast.get(x).t {
            ASTNodeType::Application | ASTNodeType::Abstraction => {
                argv_strs.push(format!("({})", arg_str));
            }
            _ => {
                argv_strs.push(arg_str);
            }
        }

        match ast.get(x).t {
            ASTNodeType::Literal => {}
            _ => literals_only = false,
        }
        let f_node = ast.get(f);

        match f_node.t {
            ASTNodeType::Identifier => {
                let name = f_node.get_value();

                return if let Some(label) = lt.get(&name) {
                    if let Some(reduction_arity) = label.inbuilt_reduction_arity {
                        if reduction_arity != argv.len() {
                            return None;
                        }
                    }

                    let argv_comma_str = comma_ify(argv_strs.iter().rev().cloned().collect());
                    if label.is_inbuilt() {
                        if literals_only {
                            Some(RCPair {
                                from: expr,
                                to: label.call_inbuilt(f_node, argv),
                                msg_after: format!(
                                    "Applied inbuilt {} to {}",
                                    name, &argv_comma_str
                                ),
                                msg_before: format!(
                                    "Apply inbuilt {} to {}",
                                    name, &argv_comma_str
                                ),
                            })
                        } else {
                            None
                        }
                    } else {
                        let assign = match am.get(&name) {
                            Some(a) => *a,
                            None => return None,
                        };

                        let assign_exp = ast.get_assign_exp(assign);
                        let n_args = ast.get_n_abstr_vars(assign_exp, argv.len());

                        if ast.get_n_abstr_vars(assign_exp, argv.len()).len() != argv.len() {
                            return None;
                        }

                        #[cfg(debug_assertions)]
                        let _n_abstr_vars_strs = n_args
                            .iter()
                            .map(|a| ast.to_string_sugar(*a, false))
                            .collect::<Vec<_>>();

                        // Stop it being a ready call when a pair is expected but we dont have it
                        for i in 0..argv.len() {
                            let _cmp_pair = (&argv[i].t, &ast.get(n_args[i]).t);
                            match (&argv[i].t, &ast.get(n_args[i]).t) {
                                (ASTNodeType::Pair, ASTNodeType::Pair) => {}
                                (_, ASTNodeType::Pair) => return None,
                                _ => {}
                            }
                        }

                        // let argv_ids = argv_ids.reverse();
                        let call_result = ast.do_multiple_abst_substs(assign_exp, argv_ids);

                        #[cfg(debug_assertions)]
                        let _ready_call_result_str =
                            call_result.to_string_sugar(call_result.root, false);

                        Some(RCPair {
                            from: expr,
                            to: call_result,
                            msg_after: format!("Applied function {} to {}", name, &argv_comma_str),
                            msg_before: format!("Apply function {} to {}", name, &argv_comma_str),
                        })
                    }
                } else {
                    let _x = lt.get_non_builtin_type_map();
                    None
                };
            }
            ASTNodeType::Abstraction => {
                return if !(f_node.wait_for_args && literals_only) {
                    let n_args = ast.get_n_abstr_vars(f, argv.len());

                    if argv.len() != n_args.len() {
                        return None;
                    }

                    for i in 0..argv.len() {
                        match (&argv[i].t, &ast.get(n_args[i]).t) {
                            (ASTNodeType::Pair, ASTNodeType::Pair) => {}
                            (_, ASTNodeType::Pair) => return None,
                            _ => {}
                        }
                    }

                    let argv_comma_str = comma_ify(argv_strs.iter().rev().cloned().collect());

                    let call_result = ast.do_multiple_abst_substs(f, argv_ids);

                    #[cfg(debug_assertions)]
                    let _ready_call_result_str =
                        call_result.to_string_sugar(call_result.root, false);

                    Some(RCPair {
                        from: expr,
                        to: call_result,
                        msg_after: format!("Apply abstraction to {}", &argv_comma_str),
                        msg_before: format!("Apply abstraction to {}", &argv_comma_str),
                    })
                } else {
                    None
                }
            }
            ASTNodeType::Application => {
                x = ast.get_arg(f);
                f = ast.get_func(f);
            }
            _ => return None,
        }
    }
}

pub fn find_all_redex_contraction_pairs(
    ast: &AST,
    module: Option<usize>,
    expr: usize,
    lt: &KnownTypeLabelTable,
) -> Vec<RCPair> {
    let mut pairs: Vec<RCPair> = vec![];

    #[cfg(debug_assertions)]
    let _exp_str = ast.to_string_sugar(expr, false);

    // Dont need to worry about this as main must be at the end, so everything defined in
    // the module is defined here
    let am: HashMap<String, usize> = match module {
        Some(m) => ast.get_assigns_map(m),
        None => HashMap::new(),
    };

    match ast.get(expr).t {
        ASTNodeType::Literal | ASTNodeType::Abstraction => {}
        ASTNodeType::Application => {
            let f = ast.get_func(expr);
            let x = ast.get_arg(expr);

            if let Some(inbuilt_reduction) = check_for_valid_call(ast, expr, &lt, am) {
                pairs.push(inbuilt_reduction);
            }

            pairs.extend(find_all_redex_contraction_pairs(ast, module, f, &lt));
            pairs.extend(find_all_redex_contraction_pairs(ast, module, x, &lt));
        }
        ASTNodeType::Pair => {
            let left_rcs = find_all_redex_contraction_pairs(ast, module, ast.get_first(expr), lt);
            let right_rcs = find_all_redex_contraction_pairs(ast, module, ast.get_second(expr), lt);
            pairs.extend(left_rcs);
            pairs.extend(right_rcs);
        }
        ASTNodeType::Match | ASTNodeType::Identifier => {
            if let Some(rc) = find_single_redex_contraction_pair(ast, module, expr, lt) {
                pairs.push(rc);
            }
        }
        _ => panic!("Expected expression"),
    }

    pairs
}

pub fn find_single_redex_contraction_pair(
    ast: &AST,
    module: Option<usize>,
    expr: usize,
    lt: &KnownTypeLabelTable,
) -> Option<RCPair> {
    #[cfg(debug_assertions)]
    let _exp_str = ast.to_string_sugar(expr, false);

    // Dont need to worry about this as main must be at the end, so everything defined in
    // the module is defined here
    let am: HashMap<String, usize> = match module {
        Some(m) => ast.get_assigns_map(m),
        None => HashMap::new(),
    };

    match ast.get(expr).t {
        ASTNodeType::Literal | ASTNodeType::Abstraction => None,
        ASTNodeType::Pair => {
            if let Some(left_rc) =
                find_single_redex_contraction_pair(ast, module, ast.get_first(expr), lt)
            {
                Some(left_rc)
            } else {
                find_single_redex_contraction_pair(ast, module, ast.get_second(expr), lt)
            }
        }
        ASTNodeType::Identifier => {
            let value = ast.get(expr).get_value();

            // It should not be non zero_ary func as otherwise it would be caught by the app case
            if let Some(label) = lt.get(&value) {
                return if label.is_inbuilt() {
                    // If the reduction arity isn't 0 then wth
                    if let Some(reduction_arity) = label.inbuilt_reduction_arity {
                        if reduction_arity != 0 {
                            return None;
                        }
                    }
                    let subst_result = label.call_inbuilt(&ast.get(expr), vec![]);
                    Some(RCPair {
                        from: expr,
                        to: subst_result,
                        msg_after: format!("Substituted label {}", &value),
                        msg_before: format!("Substitute label {}", &value),
                    })
                } else {
                    let assign = if let Some(assign) = am.get(&value) {
                        *assign
                    } else {
                        return None;
                    };

                    let assign_exp = ast.get_assign_exp(assign);
                    let subst_result = ast.clone_node(assign_exp);

                    Some(RCPair {
                        from: expr,
                        to: subst_result,
                        msg_after: format!("Substituted label {}", &value),
                        msg_before: format!("Substitute label {}", &value),
                    })
                };
            }
            None
        }
        ASTNodeType::Application => {
            if let Some(ready_call_reduction) = check_for_valid_call(ast, expr, &lt, am) {
                Some(ready_call_reduction)
            } else if let Some(f_rc) =
                find_single_redex_contraction_pair(ast, module, ast.get_func(expr), lt)
            {
                Some(f_rc)
            } else {
                find_single_redex_contraction_pair(ast, module, ast.get_arg(expr), lt)
            }
        }
        ASTNodeType::Match => {
            let unpack_expr = ast.get_match_unpack_pattern(expr);
            let mut refuted = vec![];
            for (i, (pattern, pattern_expr)) in ast.get_match_cases(expr).into_iter().enumerate() {
                let result = pattern_match(ast, unpack_expr, pattern);
                match result {
                    PatternMatchResult::Success(bindings) => {
                        let case_str = ast.to_string_sugar(pattern, false);
                        let mut pat_expr_cloned = ast.clone_node(pattern_expr);
                        for (var, replacement) in bindings {
                            let replacement_appended = pat_expr_cloned.append(ast, replacement);
                            let usages = pat_expr_cloned
                                .get_all_free_instances_of_var_in_exp(pat_expr_cloned.root, &var);
                            for usage in usages {
                                pat_expr_cloned
                                    .replace_references_to_node(usage, replacement_appended);
                            }
                        }
                        return Some(RCPair {
                            from: expr,
                            to: pat_expr_cloned.clone_node(pat_expr_cloned.root),
                            msg_after: format!("Matched to pattern {}", case_str),
                            msg_before: format!("Match to pattern {}", case_str),
                        });
                    }
                    PatternMatchResult::Unknown => {
                        // return if let Some(rc) = find_single_redex_contraction_pair(ast, module, unpack_expr, lt) {
                        //     let mut match_cloned = ast.clone_node(expr);
                        //     let match_expr = match_cloned.get_match_unpack_pattern(match_cloned.root);
                        //     match_cloned.do_rc_subst(match_cloned.root, &RCPair {
                        //         from: match_expr,
                        //         to: rc.to,
                        //         msg_after: String::new(),
                        //         msg_before: String::new()
                        //     });
                        //     Some(RCPair {
                        //         from: expr,
                        //         to: match_cloned,
                        //         msg_before: rc.msg_before + &format!(", and refute patterns {:?}. ", refuted),
                        //         msg_after: rc.msg_after + &format!(", and refuted patterns {:?}. ", refuted),
                        //     })
                        // } else {
                        //     None
                        // }
                        return find_single_redex_contraction_pair(ast, module, unpack_expr, lt);
                    }
                    PatternMatchResult::Refute => {
                        refuted.push(i);
                    }
                }
            }
            find_single_redex_contraction_pair(ast, module, unpack_expr, lt)
        }
        _ => None,
    }
}
