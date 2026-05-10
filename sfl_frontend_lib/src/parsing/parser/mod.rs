use super::lexer::{Lexer, LexerError};
use super::token::*;
use crate::ast::AST;
use crate::{ASTNodeType, KnownTypeLabelTable, Type, PRELUDE};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, prelude::*};

mod parse_expression;
mod parse_match;
mod parse_types;

pub struct Parser {
    t_queue: VecDeque<Token>,
    lexer: Lexer,
    type_assignment_map: HashMap<String, Type>,
    bound: HashSet<String>,
}

pub struct ParserError {
    pub e: String,
    line: usize,
    col: usize,
}

pub struct TypeMap {
    pub types: HashMap<String, Type>,
}

impl TypeMap {
    pub fn new() -> Self {
        let mut type_decls = HashMap::new();
        type_decls.insert("Int".to_string(), Type::int64());
        type_decls.insert("Float".to_string(), Type::float64());
        type_decls.insert("Bool".to_string(), Type::bool());
        Self { types: type_decls }
    }
}

pub struct ParseResult {
    pub ast: AST,
    pub lt: KnownTypeLabelTable,
    pub tm: TypeMap,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Parser Error at [{}:{}]: {}",
            self.line + 1,
            self.col + 1,
            self.e
        )
    }
}

impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        Self {
            e: value.e,
            line: value.line,
            col: value.col,
        }
    }
}

impl Parser {
    pub fn from_file(filename: String) -> Result<Self, io::Error> {
        let mut file = File::open(&filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(contents, Some(filename)),
            bound: HashSet::new(),
            type_assignment_map: HashMap::new(),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound: KnownTypeLabelTable::get_starting_bindings_map()
                .iter()
                .cloned()
                .collect(),
            type_assignment_map: HashMap::new(),
        }
    }

    pub fn add_bindings_from(&mut self, other: &Parser) {
        self.bound.extend(other.bound.clone());
    }

    pub fn bind(&mut self, name: String) {
        self.bound.insert(name);
    }

    pub fn bind_node(&mut self, ast: &mut AST, node: usize) -> Result<(), ParserError> {
        let n = ast.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                let str = n.get_value().clone();
                if self.bound.contains(str.as_str()) {
                    return Err(self.parse_error(format!(
                        "Variable {} is already bound, and cannot be rebound for abstraction",
                        str
                    )));
                }
                if str != "_" {
                    self.bind(str);
                }
                Ok(())
            }
            ASTNodeType::Pair => {
                self.bind_node(ast, ast.get_first(node))?;
                self.bind_node(ast, ast.get_second(node))?;
                Ok(())
            }
            _ => panic!("cant bind node"),
        }
    }

    pub fn unbind_node(&mut self, ast: &mut AST, node: usize) {
        let n = ast.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                let str = n.get_value();
                if str != "_" {
                    self.unbind(&str);
                }
            }
            ASTNodeType::Pair => {
                self.unbind_node(ast, ast.get_first(node));
                self.unbind_node(ast, ast.get_second(node));
            }
            _ => panic!("cant bind node"),
        }
    }

    pub fn unbind(&mut self, name: &String) {
        self.bound.remove(name);
    }

    fn parse_error(&self, msg: String) -> ParserError {
        ParserError {
            e: msg,
            line: self.lexer.line,
            col: self.lexer.col,
        }
    }

    // Add tk to queue
    #[inline(always)]
    fn queue_tk(&mut self) -> Result<(), ParserError> {
        let t = self.lexer.get_token()?;
        self.t_queue.push_back(t);

        Ok(())
    }

    // Roll queue forwards
    #[inline(always)]
    fn advance(&mut self) {
        self.t_queue.pop_front();
    }

    // Get nth token without advancing
    fn peek(&mut self, n: usize) -> Result<Token, ParserError> {
        while n >= self.t_queue.len() {
            self.queue_tk()?;
        }
        Ok(self.t_queue[n].clone())
    }

    // Get 0th token and advance
    #[inline(always)]
    fn consume(&mut self) -> Result<Token, ParserError> {
        let peek_result = self.peek(0);
        self.advance();
        peek_result
    }

    fn init_parser(&mut self, with_prelude: bool) -> (KnownTypeLabelTable, TypeMap, AST) {
        if with_prelude {
            let mut parser = Self::from_string(PRELUDE.to_string());
            let pr = match parser.parse_module(false) {
                Ok(pr) => pr,
                Err(e) => panic!("Failed to parse prelude: {:?}", e),
            };
            for binding in parser.bound {
                self.bind(binding);
            }
            (pr.lt, pr.tm, pr.ast)
        } else {
            let mut ast = AST::new();
            let module = ast.add_module(Vec::new(), self.lexer.line, self.lexer.col);
            ast.root = module;
            (KnownTypeLabelTable::new(), TypeMap::new(), ast)
        }
    }

    fn parse_assignment(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let ass_tk = self.peek(0)?;
        assert!(ass_tk.tt == TokenType::Id);

        let name = ass_tk.value.clone();

        if self.bound.contains(name.as_str()) {
            return Err(self.parse_error(format!("Variable already assigned: {}", name)));
        }

        self.bind(name.clone());

        self.advance();

        let t = self.peek(0)?;
        let expr = match t.tt {
            TokenType::Assignment => {
                self.advance();
                self.parse_expression(ast, type_table)?
            }
            TokenType::Id | TokenType::LParen => {
                let (expr, abst_vars) = self.parse_abstraction(ast, true, type_table)?;
                for var in abst_vars.into_iter().rev() {
                    ast.fancy_assign_abst_syntax(var);
                }
                expr
            }
            _ => {
                return Err(self.parse_error(format!("Unexpected token in assignment: {}", t.value)))
            }
        };

        let id = ast.add_id(ass_tk, self.lexer.line, self.lexer.col);

        // Ignore if type assignment is not found, so the typechecker will have to infer
        let type_assignment = match self.get_type_assignment(&name) {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        Ok(ast.add_assignment(id, expr, self.lexer.line, self.lexer.col, type_assignment))
    }

    pub fn parse_module(&mut self, with_prelude: bool) -> Result<ParseResult, ParserError> {
        let (mut lt, mut tm, mut ast) = self.init_parser(with_prelude);
        let module = ast.root;
        let mut main_found = false;

        'assloop: loop {
            let t = self.peek(0)?;

            match t.tt {
                TokenType::Id => {
                    let next = self.peek(1)?;
                    match next.tt {
                        TokenType::Assignment | TokenType::Id | TokenType::LParen => {
                            let assignment = self.parse_assignment(&mut ast, &tm.types)?;
                            let ass_node = ast.get(assignment);
                            let ass_name = ast.get_assignee(assignment);
                            if let Some(ass_type) = &ass_node.type_assignment {
                                lt.add(ass_name.clone(), ass_type.clone())
                            } else {
                                lt.add_no_type(ass_name.clone());
                            }
                            if ass_name == "main" {
                                main_found = true;
                            }
                            ast.add_to_module(module, assignment);
                        }
                        TokenType::DoubleColon => self.parse_type_assignment(&tm.types)?,
                        _ => {
                            return Err(self.parse_error(format!(
                                "Unexpected Token: {:?}. Expected assignment operator: =",
                                next.value
                            )))
                        }
                    }
                }
                TokenType::KWType => {
                    let (decl_name, decl_type) = self.parse_type_alias_decl(&tm.types)?;
                    if let Some(_) = tm.types.get(&decl_name) {
                        return Err(self
                            .parse_error(format!("Type {} declared more than once", &decl_name)));
                    }
                    #[cfg(debug_assertions)]
                    let _decl_type_str = decl_type.to_string();

                    tm.types.insert(
                        decl_name.clone(),
                        Type::Alias(decl_name, Box::new(decl_type)),
                    );
                }
                TokenType::KWData => {
                    let constructors = self.parse_data_decl(&mut tm.types)?;

                    for (constructor_name, constructor_type) in constructors {
                        lt.add(constructor_name.clone(), constructor_type);
                        self.bind(constructor_name);
                    }
                }
                TokenType::Newline => {
                    self.advance();
                }
                TokenType::EOF => {
                    break 'assloop;
                }
                _ => return Err(self.parse_error(format!("Unexpected Token: {:?}", t))),
            }
        }

        if with_prelude && !main_found {
            return Err(self.parse_error(
                "Assignment to 'main' is missing. This is the programs entry point.".to_string(),
            ));
        }

        Ok(ParseResult { ast, lt, tm })
    }

    #[cfg(test)]
    pub fn parse_tl_expression(&mut self, with_prelude: bool) -> Result<ParseResult, ParserError> {
        let (lt, tm, mut ast) = self.init_parser(with_prelude);
        ast.root = self.parse_expression(&mut ast, &tm.types)?;
        Ok(ParseResult { lt, tm, ast })
    }
}
