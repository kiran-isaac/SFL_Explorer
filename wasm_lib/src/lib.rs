mod utils;

#[cfg(target_arch = "wasm32")]
mod wasm_only {
    use super::utils::*;
    use sfl_frontend_lib::*;

    use std::collections::BTreeMap;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct ParseOptions {
        typed: bool,
        prelude: bool,
    }

    #[wasm_bindgen]
    impl ParseOptions {
        #[wasm_bindgen(constructor)]
        pub fn new(typed: bool, prelude: bool) -> Self {
            ParseOptions { typed, prelude }
        }
    }

    #[wasm_bindgen]
    pub struct RawASTInfo {
        pub ast: *mut AST,
        pub lt: *mut KnownTypeLabelTable,
        pub main_expr: usize,
    }

    #[wasm_bindgen]
    impl RawASTInfo {
        #[wasm_bindgen]
        pub unsafe fn free(&self) {
            if !self.ast.is_null() {
                drop(Box::from_raw(self.ast));
            }
            if !self.lt.is_null() {
                drop(Box::from_raw(self.lt));
            }
        }
    }

    #[wasm_bindgen]
    pub struct RawRC {
        pub from_str: *mut String,
        pub to_str: *mut String,
        pub msg1: *mut String,
        pub msg2: *mut String,
        pub(crate) redex: *mut RCPair,
    }

    impl RawRC {
        pub unsafe fn free(&self) {
            if !&self.from_str.is_null() {
                drop(Box::from_raw(*&self.from_str));
            }
            if !&self.to_str.is_null() {
                drop(Box::from_raw(*&self.to_str));
            }
            if !&self.redex.is_null() {
                drop(Box::from_raw(*&self.redex));
            }
        }
    }

    pub unsafe fn get_rc_from(rc: &RawRC) -> String {
        (&*rc.from_str).clone()
    }

    pub unsafe fn get_rc_to(rc: &RawRC) -> String {
        (&*rc.to_str).clone()
    }

    pub unsafe fn get_rc_msg2(rc: &RawRC) -> String {
        (&*rc.msg2).clone()
    }

    pub unsafe fn get_rc_msg1(rc: &RawRC) -> String {
        (&*rc.msg1).clone()
    }

    #[wasm_bindgen]
    pub unsafe fn get_rcs_from(rcs: *mut Vec<RawRC>, rc: usize) -> String {
        let rcs = &*rcs;
        get_rc_from(&rcs[rc])
    }

    #[wasm_bindgen]
    pub unsafe fn get_rcs_to(rcs: *mut Vec<RawRC>, rc: usize) -> String {
        let rcs = &*rcs;
        get_rc_to(&rcs[rc])
    }

    #[wasm_bindgen]
    pub unsafe fn get_rcs_msg2(rcs: *mut Vec<RawRC>, rc: usize) -> String {
        let rcs = &*rcs;
        get_rc_msg2(&rcs[rc])
    }

    #[wasm_bindgen]
    pub unsafe fn get_rcs_msg1(rcs: *mut Vec<RawRC>, rc: usize) -> String {
        let rcs = &*rcs;
        get_rc_msg1(&rcs[rc])
    }

    #[wasm_bindgen]
    pub unsafe fn get_rcs_len(rcs: *mut Vec<RawRC>) -> usize {
        let rcs = &*rcs;
        rcs.len()
    }

    #[wasm_bindgen]
    pub unsafe fn pick_rc_and_free(
        info: &mut RawASTInfo,
        rcs: *mut Vec<RawRC>,
        to_subst: usize,
    ) -> RawASTInfo {
        let rcs = &*rcs;

        let ast = &mut *info.ast;
        let lt = &*info.lt;
        let main_expr = info.main_expr;
        let mut rust_rcs = vec![];

        for rc in rcs {
            rust_rcs.push(&*rc.redex);
        }

        let mut ast2 = ast.clone();
        ast2.do_rc_subst(main_expr, &*rcs[to_subst].redex);

        for rc in rcs {
            rc.free();
        }

        // clone to cleanup and remove orphan nodes
        let ast2 = ast2.clone_node(ast2.root);

        let main_expr = ast2.get_assign_exp(if let Some(main) = ast2.get_main(ast2.root) {
            main
        } else {
            panic!("no main, should have been caught by parser")
        });

        RawASTInfo {
            ast: Box::into_raw(Box::new(ast2)),
            lt: Box::into_raw(Box::new(lt.clone())),
            main_expr,
        }
    }

    #[wasm_bindgen]
    pub unsafe fn get_all_redexes(info: &RawASTInfo) -> *mut Vec<RawRC> {
        let info = info;
        let ast = &mut *info.ast;
        let lt = &*info.lt;
        let module = ast.root;
        let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
        let main_expr = ast.get_assign_exp(main_assign);
        let mut rcs_output: Vec<RawRC> = vec![];
        let rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
        for rc in ast.filter_identical_rcs(&rcs) {
            let from_str = Box::into_raw(Box::new(ast.to_string_sugar(rc.from, false).clone()));
            let to_str = Box::into_raw(Box::new(rc.to.to_string_sugar(rc.to.root, false).clone()));
            let msg1 = Box::into_raw(Box::new(rc.msg_before.to_string()));
            let msg2 = Box::into_raw(Box::new(rc.msg_after.to_string()));
            rcs_output.push(RawRC {
                from_str,
                to_str,
                msg1,
                msg2,
                redex: Box::into_raw(Box::new(rc)),
            });
        }
        Box::into_raw(Box::new(rcs_output))
    }

    #[wasm_bindgen]
    pub unsafe fn get_one_redex(info: &RawASTInfo) -> *mut Vec<RawRC> {
        let info = info;
        let ast = &mut *info.ast;
        let lt = &*info.lt;
        let module = ast.root;

        let main_assign = if let Some(main) = ast.get_assign_to(module, "main".to_string()) {
            main
        } else {
            return Box::into_raw(Box::new(vec![]));
        };

        let main_expr = ast.get_assign_exp(main_assign);

        Box::into_raw(Box::new(
            if let Some(rc) =
                find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt)
            {
                let from_str = Box::into_raw(Box::new(ast.to_string_sugar(rc.from, false).clone()));
                let to_str =
                    Box::into_raw(Box::new(rc.to.to_string_sugar(rc.to.root, false).clone()));
                let msg1 = Box::into_raw(Box::new(rc.msg_before.to_string()));
                let msg2 = Box::into_raw(Box::new(rc.msg_after.to_string()));
                vec![RawRC {
                    from_str,
                    to_str,
                    msg1,
                    msg2,
                    redex: Box::into_raw(Box::new(rc)),
                }]
            } else {
                vec![]
            },
        ))
    }

    #[wasm_bindgen]
    pub fn parse(str: &str, parse_options: ParseOptions) -> Result<RawASTInfo, String> {
        let pr = match Parser::from_string(str.to_string()).parse_module(parse_options.prelude) {
            Ok(ast) => ast,
            Err(e) => return Err(format!("{:?}", e)),
        };
        let mut ast = pr.ast;
        let mut lt = pr.lt;
        let tm = pr.tm;
        let module = ast.root;

        if parse_options.typed {
            match typecheck(&mut ast, module, &mut lt, &tm) {
                Ok(_) => {}
                Err(e) => return Err(format!("{:?}", e)),
            };
        }

        let main_expr = ast.get_assign_exp(if let Some(main) = ast.get_main(ast.root) {
            main
        } else {
            panic!("no main, should have been caught by parser")
        });

        Ok(RawASTInfo {
            ast: Box::into_raw(Box::new(ast)),
            lt: Box::into_raw(Box::new(lt)),
            main_expr,
        })
    }

    #[wasm_bindgen]
    pub unsafe fn to_string(info: &RawASTInfo) -> String {
        let info = info;
        let ast = &*info.ast;
        ast.to_string_sugar(ast.root, true)
    }

    #[wasm_bindgen]
    pub unsafe fn types_to_string(info: &RawASTInfo) -> String {
        let lt = &*info.lt;

        let mut capitals_map = BTreeMap::new();
        let mut lowercase_map = BTreeMap::new();

        for (name, type_) in lt.get_non_builtin_type_map() {
            let first_name_char = name.chars().next().unwrap();
            if first_name_char.is_uppercase() {
                capitals_map.insert(name, type_);
            } else {
                lowercase_map.insert(name, type_);
            }
        }

        let mut s = String::new();
        for (name, type_) in capitals_map {
            if let Some(type_) = type_ {
                s.push_str(&format!("{} :: {}\n", name, type_));
            }
        }
        s.push('\n');
        for (name, type_) in lowercase_map {
            if let Some(type_) = type_ {
                s.push_str(&format!("{} :: {}\n", name, type_));
            }
        }
        s
    }

    #[wasm_bindgen]
    pub unsafe fn main_to_string(info: &RawASTInfo) -> String {
        let info = info;
        let ast = &*info.ast;

        let main_assign = if let Some(main) = ast.get_main(ast.root) {
            main
        } else {
            return String::new();
        };

        let main_expr = ast.get_assign_exp(main_assign);
        ast.to_string_sugar(main_expr, true)
    }

    #[wasm_bindgen]
    pub fn my_init() {
        set_panic_hook();
    }

    #[wasm_bindgen]
    pub fn get_prelude() -> String {
        PRELUDE.to_string()
    }

    #[wasm_bindgen]
    pub struct RawDiff {
        diff: *mut ASTDiff,
    }

    #[wasm_bindgen]
    pub unsafe fn diff(ast1: &RawASTInfo, ast2: &RawASTInfo) -> RawDiff {
        let ast1 = &*ast1.ast;
        let ast2 = &*ast2.ast;

        let ast1_main = ast1.get_assign_exp(ast1.get_main(ast1.root).unwrap());
        let ast2_main = ast2.get_assign_exp(ast2.get_main(ast2.root).unwrap());

        RawDiff {
            diff: Box::into_raw(Box::new(AST::diff(ast1, ast2, ast1_main, ast2_main))),
        }
    }

    #[wasm_bindgen]
    pub unsafe fn get_diff_len(diff: &RawDiff) -> usize {
        let diff = &*diff.diff;
        diff.len()
    }

    #[wasm_bindgen]
    pub unsafe fn diff_is_similar(diff: &RawDiff, index: usize) -> bool {
        let diff = &*diff.diff;

        match diff.get(index).unwrap() {
            ASTDiffElem::Similar(_) => true,
            ASTDiffElem::Different(_, _) => false,
        }
    }

    #[wasm_bindgen]
    pub unsafe fn diff_get_similar(diff: &RawDiff, index: usize) -> String {
        let diff = &*diff.diff;

        match diff.get(index).unwrap() {
            ASTDiffElem::Similar(s) => s.clone(),
            ASTDiffElem::Different(_, _) => panic!("Expected similar, got different"),
        }
    }

    #[wasm_bindgen]
    pub struct StringPair {
        str1: String,
        str2: String,
    }

    #[wasm_bindgen]
    pub unsafe fn diff_get_diff(diff: &RawDiff, index: usize) -> StringPair {
        let diff = &*diff.diff;

        match diff.get(index).unwrap() {
            ASTDiffElem::Similar(s) => panic!("Expected diff, got similar {}", s),
            ASTDiffElem::Different(str1, str2) => StringPair {
                str1: str1.clone(),
                str2: str2.clone(),
            },
        }
    }

    #[wasm_bindgen]
    pub fn stringpair_one(stringpair: &StringPair) -> String {
        stringpair.str1.clone()
    }

    #[wasm_bindgen]
    pub fn stringpair_two(stringpair: &StringPair) -> String {
        stringpair.str2.clone()
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_only::*;
