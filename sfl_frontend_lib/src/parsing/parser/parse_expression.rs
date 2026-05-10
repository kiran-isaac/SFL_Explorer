use super::*;

impl Parser {
    // Parse potentially multiple abstraction, return the abstr node and all the absts as a vector
    pub(super) fn parse_abstraction(
        &mut self,
        ast: &mut AST,
        is_assign: bool,
        type_table: &HashMap<String, Type>,
    ) -> Result<(usize, Vec<usize>), ParserError> {
        let mut args = vec![];

        loop {
            let t = self.peek(0)?;
            match (t.tt, is_assign) {
                (TokenType::Id | TokenType::LParen, _) => {
                    args.push(self.parse_abstr_var(ast, type_table)?);
                }
                (TokenType::Dot, false) => break,
                (TokenType::Assignment, true) => break,
                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in lambda argument: {}", t.value)))
                }
            }
        }

        if is_assign {
            assert_eq!(self.consume()?.tt, TokenType::Assignment);
        } else {
            assert_eq!(self.consume()?.tt, TokenType::Dot);
        }

        for arg in &args {
            match self.bind_node(ast, *arg) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        let mut expr = self.parse_expression(ast, type_table)?;

        let mut absts_vec = vec![];
        for &&arg in &args.iter().rev().collect::<Vec<&usize>>() {
            expr = ast.add_abstraction(arg, expr, self.lexer.line, self.lexer.col);
            absts_vec.push(expr);
            self.unbind_node(ast, arg);
        }
        Ok((expr, absts_vec))
    }

    fn parse_abstr_var(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let left = self.parse_abstr_var_primary(ast, type_table)?;
        match self.peek(0)?.tt {
            TokenType::Comma => {
                self.advance();
                let right = self.parse_abstr_var(ast, type_table)?;
                Ok(ast.add_pair(left, right, self.lexer.line, self.lexer.col))
            }
            TokenType::DoubleColon => {
                self.advance();
                let type_ = self.parse_type_expression(type_table, None)?;
                ast.set_type(left, type_);
                Ok(left)
            }
            TokenType::RParen => {
                self.advance();
                Ok(left)
            }
            _ => Ok(left),
        }
    }

    fn parse_abstr_var_primary(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => Ok(ast.add_id(t, self.lexer.line, self.lexer.col)),
            TokenType::LParen => self.parse_abstr_var(ast, type_table),
            _ => Err(self.parse_error("Expected identifier (or '(') after lambda".to_string())),
        }
    }

    // Parse a primary expression
    fn parse_expr_primary(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let t = self.consume()?;
        match t.tt {
            TokenType::Id | TokenType::UppercaseId => {
                let id_name = t.value.clone();
                if !self.bound.contains(&id_name) {
                    return Err(self.parse_error(format!("Unbound identifier: {}", id_name)));
                }
                Ok(ast.add_id(t, line, col))
            }
            TokenType::IntLit | TokenType::FloatLit | TokenType::BoolLit | TokenType::CharLit => {
                Ok(ast.add_lit(t, line, col))
            }
            TokenType::Match => Ok(self.parse_match(ast, type_table)?),
            TokenType::Lambda => {
                self.advance();
                Ok(self.parse_abstraction(ast, false, type_table)?.0)
            }
            TokenType::LParen | TokenType::Dollar => {
                let exp = self.parse_expression(ast, type_table)?;
                self.advance();
                Ok(exp)
            }
            _ => Err(self.parse_error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    pub(super) fn parse_expression(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let mut left = self.parse_expr_primary(ast, type_table)?;

        #[cfg(debug_assertions)]
        let _t_queue = format!("{:?}", self.t_queue);
        loop {
            #[cfg(debug_assertions)]
            let _left_str = format!("{:?}", ast.to_string_sugar(left, false));

            let line = self.lexer.line;
            let col = self.lexer.col;
            let tk = self.peek(0)?;
            match &tk.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    match self.peek(0)?.tt {
                        TokenType::RParen => {
                            self.advance();
                        }
                        _ => {
                            return Err(self.parse_error(format!(
                                "Expected closing parenthesis, got \"{:?}\"",
                                tk
                            )))
                        }
                    }
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Dollar => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    match self.peek(0)?.tt {
                        TokenType::RParen => {
                            return Err(self.parse_error(format!("Unexpected closing parenthesis")))
                        }
                        _ => {}
                    }
                    left = ast.add_app(left, right, line, col, true);
                }

                TokenType::RParen
                | TokenType::EOF
                | TokenType::Newline
                | TokenType::DoubleColon
                | TokenType::LBrace => {
                    return Ok(left);
                }

                TokenType::Comma => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    left = ast.add_pair(left, right, line, col);
                }

                TokenType::Lambda => {
                    self.advance();
                    self.parse_abstraction(ast, false, type_table)?.0;
                }

                TokenType::Match => {
                    self.advance();
                    let match_ = self.parse_match(ast, type_table)?;
                    left = ast.add_app(left, match_, line, col, false);
                }

                TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self.parse_expr_primary(ast, type_table)?;
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Id | TokenType::UppercaseId => {
                    if self.peek(0)?.is_infix_id() {
                        let id_node = self.parse_expr_primary(ast, type_table)?;
                        let right = self.parse_expression(ast, type_table)?;
                        left = ast.add_app(id_node, left, line, col, false);
                        left = ast.add_app(left, right, line, col, false);
                    } else {
                        let id_node = self.parse_expr_primary(ast, type_table)?;
                        left = ast.add_app(left, id_node, line, col, false);
                    }
                }

                _ => {
                    let e = format!("Unexpected token in expression: {:?}", self.peek(0)?);
                    return Err(self.parse_error(e));
                }
            }
        }
    }
}
