/// 类 Rust 语言的递归下降语法分析器
///
/// 已实现的文法规则（已消除左递归）：
///   0.1 变量属性, 0.2 类型, 0.3 左值
///   1.1-1.5 基础程序/语句/返回/形参/函数输出
///   2.0-2.3 变量声明/赋值
///   3.1-3.5 表达式(比较/加减/乘除/函数调用)
///   4.1-4.3 选择结构(if/else/else if)
///   5.0-5.4 循环(while/for/loop/break/continue)
///   6.1-6.4 不可变/引用/借用
///   7.0-7.4 函数表达式块/选择表达式/循环表达式
///   8.1-8.3 数组
///   9.1-9.3 元组

use crate::token::{Token, TokenKind};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// 作用域栈：存储 (是否可变, 类型)
    scopes: Vec<std::collections::HashMap<String, (bool, Type)>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, scopes: vec![std::collections::HashMap::new()] }
    }

    // ---- 辅助函数 ----

    fn peek(&self) -> &TokenKind {
        &self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof)
    }

    fn peek_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token { kind: TokenKind::Eof, line: 0, col: 0 })
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<(), String> {
        let tok = self.peek_token().clone();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("{}:{}: 期望 '{}', 实际为 '{}'", tok.line, tok.col, expected, tok.kind))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let tok = self.peek_token().clone();
        if let TokenKind::Ident(name) = &tok.kind {
            let name = name.clone();
            self.pos += 1;
            Ok(name)
        } else {
            Err(format!("{}:{}: 期望标识符, 实际为 '{}'", tok.line, tok.col, tok.kind))
        }
    }

    fn expect_int(&mut self) -> Result<i64, String> {
        let tok = self.peek_token().clone();
        if let TokenKind::IntLiteral(n) = &tok.kind {
            let n = *n;
            self.pos += 1;
            Ok(n)
        } else {
            Err(format!("{}:{}: 期望整数, 实际为 '{}'", tok.line, tok.col, tok.kind))
        }
    }

    fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }
 
    fn push_scope(&mut self) {
        self.scopes.push(std::collections::HashMap::new());
    }
 
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
 
    fn declare_variable(&mut self, name: String, mutable: bool, ty: Option<Type>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (mutable, ty.unwrap_or(Type::I32)));
        }
    }
 
    fn lookup_variable(&self, name: &str) -> Option<&(bool, Type)> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// 推断表达式类型（简单推断，用于语义校验）
    fn get_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLiteral(_) => Type::I32,
            Expr::Ident(name) => {
                if let Some((_, ty)) = self.lookup_variable(name) {
                    ty.clone()
                } else {
                    Type::I32
                }
            }
            Expr::BinaryOp(l, op, _) => {
                // 比较运算符返回布尔语义，此处简化为 i32
                match op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Type::I32,
                    _ => self.get_expr_type(l),
                }
            }
            Expr::UnaryDeref(e) => {
                if let Type::Ref(t) | Type::MutRef(t) = self.get_expr_type(e) {
                    *t
                } else {
                    Type::I32
                }
            }
            Expr::Ref(e) => Type::Ref(Box::new(self.get_expr_type(e))),
            Expr::MutRef(e) => Type::MutRef(Box::new(self.get_expr_type(e))),
            Expr::ArrayLiteral(elems) => {
                let ty = if !elems.is_empty() { self.get_expr_type(&elems[0]) } else { Type::I32 };
                Type::Array(Box::new(ty), elems.len() as i64)
            }
            Expr::TupleLiteral(elems) => {
                Type::Tuple(elems.iter().map(|e| self.get_expr_type(e)).collect())
            }
            Expr::Index(base, _) => {
                if let Type::Array(t, _) = self.get_expr_type(base) {
                    *t
                } else {
                    Type::I32
                }
            }
            Expr::TupleIndex(base, idx) => {
                if let Type::Tuple(ts) = self.get_expr_type(base) {
                    ts.get(*idx as usize).cloned().unwrap_or(Type::I32)
                } else {
                    Type::I32
                }
            }
            Expr::Paren(e) => self.get_expr_type(e),
            Expr::ExprBlock(eb) => {
                if let Some(tail) = &eb.tail_expr {
                    self.get_expr_type(tail)
                } else {
                    Type::I32 // 默认 Block 返回 i32 (此处为简化)
                }
            }
            _ => Type::I32,
        }
    }

    /// 检查表达式是否为可写的“左值”
    fn is_mutable_lvalue(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                if let Some((mutable, _)) = self.lookup_variable(name) {
                    *mutable
                } else {
                    false
                }
            }
            Expr::UnaryDeref(e) => {
                // 如果是 *ptr，检查 ptr 是否为 &mut T
                let ty = self.get_expr_type(e);
                matches!(ty, Type::MutRef(_))
            }
            Expr::Index(base, _) | Expr::TupleIndex(base, _) => {
                // 如果是数组下标或元组下标，取决于基础表达式是否可变
                self.is_mutable_lvalue(base)
            }
            Expr::Paren(e) => self.is_mutable_lvalue(e),
            _ => false,
        }
    }

    // ---- 程序入口 ----

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut declarations = Vec::new();
        while !self.at(&TokenKind::Eof) {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program { declarations })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, String> {
        Ok(Declaration::Function(self.parse_function_decl()?))
    }

    // ---- 函数声明 (1.1, 1.4, 1.5, 7.2) ----

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, String> {
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.at(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.push_scope();
        for p in &params {
            self.declare_variable(p.name.clone(), p.mutable, Some(p.ty.clone()));
        }
 
        // Try to parse as expression block body (rule 7.2) or regular block
        let body = self.parse_block()?;
        self.pop_scope();

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            is_expr_body: false,
        })
    }

    // ---- 参数列表 (1.4) ----

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if self.at(&TokenKind::RParen) {
            return Ok(params);
        }
        params.push(self.parse_param()?);
        while self.at(&TokenKind::Comma) {
            self.advance();
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, String> {
        let mutable = self.parse_mutability();
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(Param { mutable, name, ty })
    }

    // ---- 类型系统 (0.2, 6.2, 6.3, 8.1, 9.1) ----

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().clone() {
            TokenKind::I32 => {
                self.advance();
                Ok(Type::I32)
            }
            TokenKind::Ampersand => {
                self.advance();
                if self.at(&TokenKind::Mut) {
                    self.advance();
                    let inner = self.parse_type()?;
                    Ok(Type::MutRef(Box::new(inner)))
                } else {
                    let inner = self.parse_type()?;
                    Ok(Type::Ref(Box::new(inner)))
                }
            }
            TokenKind::LBracket => {
                // [T; N]
                self.advance();
                let inner = self.parse_type()?;
                self.expect(&TokenKind::Semicolon)?;
                let size = self.expect_int()?;
                self.expect(&TokenKind::RBracket)?;
                Ok(Type::Array(Box::new(inner), size))
            }
            TokenKind::LParen => {
                // tuple type (T1, T2, ...)
                self.advance();
                if self.at(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Type::Tuple(vec![]));
                }
                let first = self.parse_type()?;
                if self.at(&TokenKind::Comma) {
                    self.advance();
                    let mut types = vec![first];
                    if !self.at(&TokenKind::RParen) {
                        types.push(self.parse_type()?);
                        while self.at(&TokenKind::Comma) {
                            self.advance();
                            types.push(self.parse_type()?);
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Type::Tuple(types))
                } else {
                    self.expect(&TokenKind::RParen)?;
                    Ok(Type::Tuple(vec![first]))
                }
            }
            _ => {
                let tok = self.peek_token().clone();
                Err(format!("{}:{}: 期望类型, 实际为 '{}'", tok.line, tok.col, tok.kind))
            }
        }
    }

    // ---- 可变性属性 (0.1, 6.1) ----

    fn parse_mutability(&mut self) -> bool {
        if self.at(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ---- 语句块 (1.1) ----

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(&TokenKind::LBrace)?;
        self.push_scope();
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            stmts.push(self.parse_statement()?);
        }
        self.expect(&TokenKind::RBrace)?;
        self.pop_scope();
        Ok(Block { stmts })
    }

    // ---- 表达式块 (7.0) ----

    fn parse_expr_block(&mut self) -> Result<ExprBlock, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();

        loop {
            if self.at(&TokenKind::RBrace) || self.at(&TokenKind::Eof) {
                break;
            }

            // Try to see if the next thing is a statement or a tail expression
            // A tail expression is an expression at the end without a semicolon

            // Check if it looks like a statement prefix
            if self.is_statement_start() {
                stmts.push(self.parse_statement()?);
            } else {
                // Try parsing as expression
                let expr = self.parse_expression()?;
                if self.at(&TokenKind::Semicolon) {
                    self.advance();
                    stmts.push(Statement::ExprStmt(expr));
                } else if self.at(&TokenKind::RBrace) {
                    // This is the tail expression
                    self.expect(&TokenKind::RBrace)?;
                    return Ok(ExprBlock { stmts, tail_expr: Some(Box::new(expr)) });
                } else {
                    let tok = self.peek_token().clone();
                    return Err(format!("{}:{}: 函数表达式块中期望 ';' 或 '}}', 实际为 '{}'", tok.line, tok.col, tok.kind));
                }
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(ExprBlock { stmts, tail_expr: None })
    }

    /// Check if current position looks like a clear statement start
    fn is_statement_start(&self) -> bool {
        matches!(self.peek(),
            TokenKind::Let | TokenKind::Return | TokenKind::If
            | TokenKind::While | TokenKind::For | TokenKind::Loop
            | TokenKind::Break | TokenKind::Continue | TokenKind::Semicolon
        )
    }

    // ---- 语句解析 (1.2, 1.3, 2.1-2.3, 3.1, 4.1, 5.0-5.4) ----

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek().clone() {
            TokenKind::Semicolon => {
                self.advance();
                Ok(Statement::Empty)
            }
            TokenKind::Return => {
                self.advance();
                if self.at(&TokenKind::Semicolon) {
                    self.advance();
                    Ok(Statement::Return(None))
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::Return(Some(expr)))
                }
            }
            TokenKind::Let => {
                self.advance();
                let var_decl = self.parse_var_decl()?;
                if self.at(&TokenKind::Assign) {
                    self.advance();
                    let expr = self.parse_expression()?;
                    
                    // 类型一致性检查已移交给语义分析器 (Compiler)
                    
                    self.declare_variable(var_decl.name.clone(), var_decl.mutable, var_decl.ty.clone());
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::VarDeclAssign(var_decl, expr))
                } else {
                    self.declare_variable(var_decl.name.clone(), var_decl.mutable, var_decl.ty.clone());
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::VarDecl(var_decl))
                }
            }
            TokenKind::If => {
                let if_stmt = self.parse_if_stmt()?;
                Ok(Statement::If(if_stmt))
            }
            TokenKind::While => {
                self.advance();
                let cond = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::While(cond, body))
            }
            TokenKind::For => {
                let tok = self.peek_token().clone();
                self.advance();
                let var = self.parse_for_var()?;
                self.expect(&TokenKind::In)?;
                let iter_expr = self.parse_expression()?;
                

                // Scope for loop body
                self.expect(&TokenKind::LBrace)?;
                self.push_scope();
                self.declare_variable(var.name.clone(), var.mutable, var.ty.clone());
                let mut stmts = Vec::new();
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    stmts.push(self.parse_statement()?);
                }
                self.expect(&TokenKind::RBrace)?;
                self.pop_scope();
                Ok(Statement::For(var, iter_expr, Block { stmts }))
            }
            TokenKind::Loop => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::Loop(body))
            }
            TokenKind::Break => {
                self.advance();
                if self.at(&TokenKind::Semicolon) {
                    self.advance();
                    Ok(Statement::Break(None))
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::Break(Some(expr)))
                }
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Continue)
            }
            _ => {
                // Could be assignment or expression statement
                let expr = self.parse_expression()?;
                if self.at(&TokenKind::Assign) {
                    // 赋值语句: lvalue = expr ;
                    let tok = self.peek_token().clone();
                    self.advance();
                    
                    // 校验已移交至语义分析器
                    let rhs = self.parse_expression()?;
                    
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::Assign(expr, rhs))
                } else {
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(Statement::ExprStmt(expr))
                }
            }
        }
    }

    // ---- 变量声明 (2.0) ----

    fn parse_var_decl(&mut self) -> Result<VarDecl, String> {
        let mutable = self.parse_mutability();
        let name = self.expect_ident()?;
        let ty = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok(VarDecl { mutable, name, ty })
    }

    fn parse_for_var(&mut self) -> Result<VarDeclName, String> {
        let mutable = self.parse_mutability();
        let name = self.expect_ident()?;
        let ty = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok(VarDeclName { mutable, name, ty })
    }

    // ---- 选择结构 (4.1-4.3) ----

    fn parse_if_stmt(&mut self) -> Result<IfStmt, String> {
        self.expect(&TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_part = if self.at(&TokenKind::Else) {
            self.advance();
            if self.at(&TokenKind::If) {
                // else if
                let elif = self.parse_if_stmt()?;
                Some(ElsePart::ElseIf(Box::new(elif)))
            } else {
                let else_block = self.parse_block()?;
                Some(ElsePart::ElseBlock(else_block))
            }
        } else {
            None
        };

        Ok(IfStmt {
            condition,
            then_block,
            else_part,
        })
    }

    // ---- 表达式解析 (3.1-3.5) ----
    // Precedence (low to high):
    //   Range (..)
    //   Comparison (==, !=, <, <=, >, >=)
    //   Add/Sub (+, -)
    //   Mul/Div (*, /)
    //   Unary (*, &, &mut)
    //   Postfix (call, index, tuple index)
    //   Primary (int, ident, paren, array, tuple, block, if-expr, loop-expr)

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_range_expr()
    }

    fn parse_range_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_comparison()?;
        if self.at(&TokenKind::DotDot) {
            self.advance();
            let right = self.parse_comparison()?;
            Ok(Expr::Range(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_add_sub()?;
        loop {
            let op = match self.peek() {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_add_sub()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul_div()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_div()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            TokenKind::Star => {
                // Dereference (*expr)  rule 6.4
                self.advance();
                let inner = self.parse_unary()?;
                Ok(Expr::UnaryDeref(Box::new(inner)))
            }
            TokenKind::Ampersand => {
                self.advance();
                if self.at(&TokenKind::Mut) {
                    // &mut expr  rule 6.3
                    self.advance();
                    let inner = self.parse_unary()?;
                    Ok(Expr::MutRef(Box::new(inner)))
                } else {
                    // &expr  rule 6.2
                    let inner = self.parse_unary()?;
                    Ok(Expr::Ref(Box::new(inner)))
                }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek().clone() {
                TokenKind::LBracket => {
                    // index: expr[expr]  rule 8.3
                    self.advance();
                    let idx = self.parse_expression()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Index(Box::new(expr), Box::new(idx));
                }
                TokenKind::Dot => {
                    // tuple index: expr.N  rule 9.3
                    self.advance();
                    let n = self.expect_int()?;
                    expr = Expr::TupleIndex(Box::new(expr), n);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            TokenKind::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            TokenKind::Ident(name) => {
                self.advance();
                // Check for function call  rule 3.5
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            TokenKind::LParen => {
                self.advance();
                // Could be: (expr), tuple (expr,expr,...), or empty tuple ()
                if self.at(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr::TupleLiteral(vec![]));
                }
                let first = self.parse_expression()?;
                if self.at(&TokenKind::Comma) {
                    // Tuple literal  rule 9.2
                    self.advance();
                    let mut elems = vec![first];
                    if !self.at(&TokenKind::RParen) {
                        elems.push(self.parse_expression()?);
                        while self.at(&TokenKind::Comma) {
                            self.advance();
                            if self.at(&TokenKind::RParen) { break; }
                            elems.push(self.parse_expression()?);
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::TupleLiteral(elems))
                } else {
                    // Parenthesized expression
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Paren(Box::new(first)))
                }
            }
            TokenKind::LBracket => {
                // Array literal  rule 8.2
                self.advance();
                let mut elems = Vec::new();
                if !self.at(&TokenKind::RBracket) {
                    elems.push(self.parse_expression()?);
                    while self.at(&TokenKind::Comma) {
                        self.advance();
                        if self.at(&TokenKind::RBracket) { break; }
                        elems.push(self.parse_expression()?);
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::ArrayLiteral(elems))
            }
            TokenKind::LBrace => {
                // Expression block  rule 7.0/7.1
                let eb = self.parse_expr_block()?;
                Ok(Expr::ExprBlock(eb))
            }
            TokenKind::If => {
                // If expression  rule 7.3
                self.advance();
                let cond = self.parse_expression()?;
                let then_block = self.parse_expr_block()?;
                self.expect(&TokenKind::Else)?;
                let else_block = self.parse_expr_block()?;
                Ok(Expr::IfExpr(Box::new(cond), Box::new(then_block), Box::new(else_block)))
            }
            TokenKind::Loop => {
                // Loop expression  rule 7.4
                self.advance();
                let body = self.parse_block()?;
                Ok(Expr::LoopExpr(body))
            }
            _ => {
                let tok = self.peek_token().clone();
                Err(format!("{}:{}: 期望表达式, 实际为 '{}'", tok.line, tok.col, tok.kind))
            }
        }
    }

    // ---- 参数列表 (3.5) ----

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.at(&TokenKind::RParen) {
            return Ok(args);
        }
        args.push(self.parse_expression()?);
        while self.at(&TokenKind::Comma) {
            self.advance();
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}
