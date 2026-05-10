use super::*;

impl AST {
    pub fn get_all_free_instances_of_var_in_exp(&self, exp: usize, var: &String) -> Vec<usize> {
        match self.get(exp).t {
            ASTNodeType::Literal => {
                vec![]
            }
            ASTNodeType::Identifier => {
                if var == &self.get(exp).get_value() {
                    vec![exp]
                } else {
                    vec![]
                }
            }
            ASTNodeType::Application => {
                let mut left = self.get_all_free_instances_of_var_in_exp(self.get_func(exp), &var);
                let right = self.get_all_free_instances_of_var_in_exp(self.get_arg(exp), &var);
                left.extend(right);
                left
            }
            ASTNodeType::Abstraction => {
                // If equal then it will not be free in abst expression so dont bother
                let abstr2_var = self.get_abstr_var(exp);
                let abstr2_var_n = self.get(abstr2_var);
                let subst_in_abstr = match abstr2_var_n.t {
                    ASTNodeType::Pair => {
                        let first_name = &self.get(self.get_first(abstr2_var)).get_value();
                        let second_name = &self.get(self.get_second(abstr2_var)).get_value();
                        first_name != var && second_name != var
                    }
                    ASTNodeType::Identifier => {
                        let name = &self.get(self.get_abstr_var(exp)).get_value();
                        name != var
                    }
                    _ => unreachable!("abst arg is not a pair or an identifier??"),
                };
                if subst_in_abstr {
                    self.get_all_free_instances_of_var_in_exp(self.get_abstr_expr(exp), var)
                } else {
                    vec![]
                }
            }
            ASTNodeType::Pair => {
                let mut left = self.get_all_free_instances_of_var_in_exp(self.get_first(exp), &var);
                let right = self.get_all_free_instances_of_var_in_exp(self.get_second(exp), &var);
                left.extend(right);
                left
            }
            ASTNodeType::Match => {
                let thing_being_matched = self.get_match_unpack_pattern(exp);
                let mut instances =
                    self.get_all_free_instances_of_var_in_exp(thing_being_matched, &var);

                for (pattern, expr) in self.get_match_cases(exp) {
                    assert!(self
                        .get_all_free_instances_of_var_in_exp(pattern, &var)
                        .is_empty());

                    instances.extend(self.get_all_free_instances_of_var_in_exp(expr, &var));
                }

                instances
            }
            _ => panic!("Cannot find var instances in non exp"),
        }
    }

    pub fn get_abst_var_usages(&self, abst: usize) -> Vec<usize> {
        let var_name = self.get(self.get_abstr_var(abst)).get_value();
        self.get_all_free_instances_of_var_in_exp(self.get_abstr_expr(abst), &var_name)
    }

    pub fn get_n_abstr_vars(&self, abstr: usize, n: usize) -> Vec<usize> {
        if n <= 0 || self.get(abstr).t != ASTNodeType::Abstraction {
            vec![]
        } else {
            let var = self.get_abstr_var(abstr);
            let mut expr = self.get_n_abstr_vars(self.get_abstr_expr(abstr), n - 1);
            expr.push(var);
            expr
        }
    }

    pub fn do_multiple_abst_substs(&self, abst: usize, substs: Vec<usize>) -> Self {
        assert!(substs.len() > 0);
        if self.get(abst).t != ASTNodeType::Abstraction {
            panic!(
                "Expected abstraction, got {}",
                self.to_string_sugar(abst, false)
            );
        }

        let substs_strings = substs
            .iter()
            .map(|s| self.to_string_sugar(*s, false))
            .collect::<Vec<_>>();

        let mut abst_ast = self.do_abst_subst(abst, *substs.last().unwrap()).unwrap();
        let substs = &substs[..substs.len() - 1];
        for subst in substs.iter().rev() {
            let subst = abst_ast.append(self, *subst);
            abst_ast = match abst_ast.do_abst_subst(abst_ast.root, subst) {
                Ok(new) => new,
                Err(_) => panic!(
                    "Failed to subst {:?} into {}",
                    substs_strings,
                    self.to_string_sugar(abst, false)
                ),
            }
        }

        abst_ast
    }

    pub fn do_abst_subst(&self, abstr: usize, subst: usize) -> Result<Self, ()> {
        if self.get(abstr).t != ASTNodeType::Abstraction {
            return Err(());
        }
        let mut cloned_abst_expr = self.clone_node(abstr);
        let new_abstr_var = cloned_abst_expr.get_abstr_var(cloned_abst_expr.root);
        let subst_id = cloned_abst_expr.append(&self, subst);

        cloned_abst_expr.replace_var_usages_in_top_level_abstraction(new_abstr_var, subst_id);
        let _abst_str = cloned_abst_expr.to_string_sugar(cloned_abst_expr.root, false);
        cloned_abst_expr.root = cloned_abst_expr.get_abstr_expr(cloned_abst_expr.root);
        let _abst_str = cloned_abst_expr.to_string_sugar(cloned_abst_expr.root, false);
        Ok(cloned_abst_expr)
    }

    pub fn replace_var_usages_in_top_level_abstraction(&mut self, var: usize, subst: usize) {
        #[cfg(debug_assertions)]
        let _var_str = self.to_string_sugar(var, false);
        #[cfg(debug_assertions)]
        let _subst_str = self.to_string_sugar(subst, false);

        let var_n = self.get(var);
        match var_n.t {
            ASTNodeType::Identifier => {
                let var_name = self.get(var).get_value();
                let usages = self.get_all_free_instances_of_var_in_exp(
                    self.get_abstr_expr(self.root),
                    &var_name,
                );
                for usage in usages {
                    self.replace_references_to_node(usage, subst);
                }
            }
            ASTNodeType::Pair => {
                let subst_first = self.get_first(subst);
                let subst_second = self.get_second(subst);
                self.replace_var_usages_in_top_level_abstraction(self.get_first(var), subst_first);
                self.replace_var_usages_in_top_level_abstraction(
                    self.get_second(var),
                    subst_second,
                );
            }
            _ => panic!("WTF HOW DID THIS HAPPEN"),
        }
    }

    fn get_laziest_rc_recurse(
        &self,
        expr: usize,
        rc_map: &HashMap<usize, &RCPair>,
    ) -> Option<RCPair> {
        if rc_map.contains_key(&expr) {
            return Some(rc_map[&expr].clone());
        }

        #[cfg(debug_assertions)]
        let _expr_str = self.to_string_sugar(expr, false);
        #[cfg(debug_assertions)]
        let _rcs_strs = {
            let mut _rcs_strs = vec![];
            for rc in rc_map.values() {
                _rcs_strs.push(self.to_string_sugar(rc.from, false));
            }
            _rcs_strs
        };

        match self.get(expr).t {
            ASTNodeType::Application | ASTNodeType::Pair => {
                let f = self.get(expr).children[0];
                let x = self.get(expr).children[1];

                if let Some(rc) = self.get_laziest_rc_recurse(f, &rc_map) {
                    return Some(rc);
                }

                self.get_laziest_rc_recurse(x, &rc_map)
            }
            _ => None,
        }
    }

    pub fn get_laziest_rc(&self, expr: usize, rcs: &Vec<RCPair>) -> Option<RCPair> {
        self.get_laziest_rc_borrowed(expr, &rcs.into_iter().map(|rc| rc).collect())
    }

    pub fn get_laziest_rc_borrowed(&self, expr: usize, rcs: &Vec<&RCPair>) -> Option<RCPair> {
        // Convert to map for O(1) lookup of whether a node is an RC
        let mut rc_map: HashMap<usize, &RCPair> = HashMap::new();
        for rc in rcs {
            rc_map.insert(rc.from, &rc);
        }

        if rcs.is_empty() {
            None
        } else {
            self.get_laziest_rc_recurse(expr, &rc_map)
        }
    }

    pub(crate) fn replace_references_to_node(&mut self, old: usize, new: usize) {
        if self.root == old {
            self.root = new;
        }

        #[cfg(debug_assertions)]
        let _ast_before = self.to_string_sugar(self.root, false);

        for n in &mut self.vec {
            for c in &mut n.children {
                if *c == old {
                    *c = new;
                }
            }
        }

        #[cfg(debug_assertions)]
        let _ast_before = self.to_string_sugar(self.root, false);
    }

    // Add a node from another ast to this ast with its children
    pub fn append(&mut self, other: &AST, node: usize) -> usize {
        let n = other.get(node);
        match n.t {
            ASTNodeType::Identifier => self.add_id(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Literal => self.add_lit(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Application => {
                let f = self.append(other, other.get_func(node));
                let x = self.append(other, other.get_arg(node));
                self.add_app(f, x, n.line, n.col, n.dollar_app)
            }
            ASTNodeType::Assignment => {
                let id = self.append(other, n.children[0]);
                let exp = self.append(other, other.get_assign_exp(node));
                self.add_assignment(id, exp, n.line, n.col, n.type_assignment.clone())
            }
            ASTNodeType::Abstraction => {
                let var = self.append(other, n.children[0]);
                let exp = self.append(other, other.get_abstr_expr(node));
                let s = self.add_abstraction(var, exp, n.line, n.col);
                if n.fancy_assign_abst_syntax {
                    self.fancy_assign_abst_syntax(s);
                }
                if n.wait_for_args {
                    self.wait_for_args(s);
                }
                s
            }
            ASTNodeType::Match => {
                let mut children = vec![];
                for a in n.children.clone() {
                    children.push(self.append(other, a));
                }
                self.add_match(children, n.line, n.col)
            }
            ASTNodeType::Module => {
                let mut assigns = vec![];
                for a in n.children.clone() {
                    assigns.push(self.append(other, a));
                }
                self.add_module(assigns, n.line, n.col)
            }
            ASTNodeType::Pair => {
                let a = self.append(other, n.children[0]);
                let b = self.append(other, n.children[1]);
                self.add_pair(a, b, n.line, n.col)
            }
        }
    }

    pub fn append_root(&mut self, other: &AST) -> usize {
        self.append(other, other.root)
    }

    fn rc_replacement_recurse(&mut self, within: usize, old: usize, new: usize) -> usize {
        #[cfg(debug_assertions)]
        let _within_str = format!("{}", self.to_string_sugar(within, false));
        #[cfg(debug_assertions)]
        let _old_str = self.to_string_sugar(old, false);
        #[cfg(debug_assertions)]
        let _new_str = self.to_string_sugar(new, false);

        if within == old {
            self.replace_references_to_node(within, new);
            return new;
        }

        if self.expr_eq(within, old) {
            self.replace_references_to_node(within, new);
            return new;
        }

        let within_n = self.get(within);

        match within_n.t {
            ASTNodeType::Application | ASTNodeType::Pair => {
                let first = within_n.children[0];
                let second = within_n.children[1];
                self.rc_replacement_recurse(first, old, new);
                self.rc_replacement_recurse(second, old, new);
            }
            ASTNodeType::Match => {
                let matched_thingy = self.get_match_unpack_pattern(within);
                self.rc_replacement_recurse(matched_thingy, old, new);
                for (_, match_case_expr) in self.get_match_cases(within) {
                    self.rc_replacement_recurse(match_case_expr, old, new);
                }
            }
            ASTNodeType::Abstraction | ASTNodeType::Literal | ASTNodeType::Identifier => {}
            _ => {
                panic!("Non expr node: {:?}", within_n)
            }
        }
        within
    }

    pub fn do_rc_subst(&mut self, within: usize, rc: &RCPair) -> usize {
        let other = &rc.to;
        let old = rc.from;
        let new = self.append(other, other.root);

        #[cfg(debug_assertions)]
        let _old_str = self.to_string_sugar(old, false);
        #[cfg(debug_assertions)]
        let _new_str = self.to_string_sugar(new, false);
        self.rc_replacement_recurse(within, old, new)
    }

    pub fn filter_identical_rcs(&self, rcs: &Vec<RCPair>) -> Vec<RCPair> {
        let mut stringset = HashSet::new();
        let mut new_rcs = vec![];
        for rc in rcs {
            let str = self.to_string_sugar(rc.from, false);
            if !stringset.contains(&str) {
                new_rcs.push(rc.clone());
            }
            stringset.insert(str);
        }
        new_rcs
    }

    pub fn do_rc_subst_and_identical_rcs(
        &mut self,
        within: usize,
        rc0: &RCPair,
        rcs: &Vec<RCPair>,
    ) {
        self.do_rc_subst_and_identical_rcs_borrowed(
            within,
            rc0,
            &rcs.into_iter().map(|rc| rc).collect(),
        );
    }

    pub fn do_rc_subst_and_identical_rcs_borrowed(
        &mut self,
        within: usize,
        rc0: &RCPair,
        rcs: &Vec<&RCPair>,
    ) {
        #[cfg(debug_assertions)]
        let _rc0_0_str = self.to_string_sugar(rc0.from, false);
        #[cfg(debug_assertions)]
        let _rc1_0_str = rc0.to.to_string_sugar(rc0.to.root, false);

        for rc in rcs {
            #[cfg(debug_assertions)]
            let _this_rc = self.to_string_sugar(rc.from, false);
            #[cfg(debug_assertions)]
            let _this_rc_1 = rc.to.to_string_sugar(rc.to.root, false);
            if self.expr_eq(rc0.from, rc.from) {
                self.do_rc_subst(within, rc);
            }
        }
    }
}
