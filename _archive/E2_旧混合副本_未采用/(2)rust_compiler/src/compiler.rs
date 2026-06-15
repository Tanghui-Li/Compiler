use crate::ast::*;
use crate::ir::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct SymbolInfo {
    mutable: bool,
    ty: Type,
    // IR Arg representing this symbol
    arg: Arg,
}

pub struct Compiler {
    scopes: Vec<HashMap<String, SymbolInfo>>,
    ir: IRProgram,
    temp_count: usize,
    label_count: usize,
    errors: Vec<String>,
    // For break/continue labels: (continue_label, break_label, return_type_if_any)
    loop_stack: Vec<(usize, usize, Option<Type>)>,
    // For function return type checking
    current_return_type: Option<Type>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            scopes: vec![HashMap::new()],
            ir: IRProgram::new(),
            temp_count: 0,
            label_count: 0,
            errors: Vec::new(),
            loop_stack: Vec::new(),
            current_return_type: None,
        }
    }

    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print_errors(&self) {
        for err in &self.errors {
            println!("[Semantic Error] {}", err);
        }
    }

    pub fn get_ir(&self) -> &IRProgram {
        &self.ir
    }

    fn new_temp(&mut self) -> Arg {
        let t = self.temp_count;
        self.temp_count += 1;
        Arg::Temp(t)
    }

    fn new_label(&mut self) -> usize {
        let l = self.label_count;
        self.label_count += 1;
        l
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: String, mutable: bool, ty: Type) -> Arg {
        let arg = Arg::Var(name.clone()); // We use the name directly for simplicity in IR
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                // Rule: 重影是允许的，但是在同一个 let 语句里通常不会
                // Rust allows shadowing in the same scope, we permit it.
            }
            scope.insert(name, SymbolInfo { mutable, ty, arg: arg.clone() });
        }
        arg
    }

    fn lookup_variable(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    // ================= AST Traversal =================

    pub fn compile_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            self.compile_declaration(decl);
        }
    }

    fn compile_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Function(f) => self.compile_function(f),
        }
    }

    fn compile_function(&mut self, f: &FunctionDecl) {
        // Emit Label for function
        let func_label = self.new_label();
        self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Var(f.name.clone()));

        self.current_return_type = f.return_type.clone();
        self.push_scope();

        for p in &f.params {
            self.declare_variable(p.name.clone(), p.mutable, p.ty.clone());
            // IR convention:PARAM
            self.ir.emit(Op::Param, Arg::Empty, Arg::Empty, Arg::Var(p.name.clone()));
        }

        self.compile_block(&f.body);

        // Implicit return at the end of function
        self.ir.emit(Op::Return, Arg::Empty, Arg::Empty, Arg::Empty);

        self.pop_scope();
        self.current_return_type = None;
    }

    fn compile_block(&mut self, block: &Block) {
        self.push_scope();
        for stmt in &block.stmts {
            self.compile_statement(stmt);
        }
        self.pop_scope();
    }

    fn compile_expr_block(&mut self, eb: &ExprBlock) -> (Type, Arg) {
        self.push_scope();
        for stmt in &eb.stmts {
            self.compile_statement(stmt);
        }
        let res = if let Some(tail) = &eb.tail_expr {
            self.compile_expression(tail)
        } else {
            (Type::Tuple(vec![]), Arg::Empty) // Unit type representation
        };
        self.pop_scope();
        res
    }

    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Empty => {}
            Statement::ExprStmt(e) => {
                self.compile_expression(e);
            }
            Statement::Return(opt_e) => {
                if let Some(e) = opt_e {
                    let (ty, arg) = self.compile_expression(e);
                    if let Some(ret_ty) = &self.current_return_type {
                        if ty != *ret_ty {
                            self.error(format!("返回语句类型 {:?} 和函数声明类型 {:?} 不一致", ty, ret_ty));
                        }
                    } else {
                        self.error(format!("返回语句类型 {:?} 和函数声明类型空不一致", ty));
                    }
                    self.ir.emit(Op::Return, arg, Arg::Empty, Arg::Empty);
                } else {
                    if let Some(ret_ty) = &self.current_return_type {
                        self.error(format!("返回语句类型空和函数声明类型 {:?} 不一致", ret_ty));
                    }
                    self.ir.emit(Op::Return, Arg::Empty, Arg::Empty, Arg::Empty);
                }
            }
            Statement::VarDecl(v) => {
                let ty = v.ty.clone().unwrap_or(Type::I32); // Default to i32 if unresolved, error reporting simplified
                self.declare_variable(v.name.clone(), v.mutable, ty);
            }
            Statement::VarDeclAssign(v, e) => {
                let (expr_ty, expr_arg) = self.compile_expression(e);
                if let Some(declared_ty) = &v.ty {
                    if *declared_ty != expr_ty {
                        self.error(format!("变量 '{}' 初始化类型不一致：期望 {:?}，实际 {:?}", v.name, declared_ty, expr_ty));
                    }
                }
                let var_arg = self.declare_variable(v.name.clone(), v.mutable, expr_ty);
                self.ir.emit(Op::Assign, expr_arg, Arg::Empty, var_arg);
            }
            Statement::Assign(lhs, rhs) => {
                let (lhs_ty, lhs_arg) = self.compile_lvalue(lhs);
                let (rhs_ty, rhs_arg) = self.compile_expression(rhs);
                if lhs_ty != rhs_ty {
                    self.error(format!("赋值类型不一致：左值 {:?}，右值 {:?}", lhs_ty, rhs_ty));
                }
                // Determine assignment operation based on whether it is an array store or pointer deref
                match lhs {
                    Expr::UnaryDeref(_) => {
                        self.ir.emit(Op::DerefAssign, rhs_arg, Arg::Empty, lhs_arg);
                    }
                    Expr::Index(base, _) => {
                        // lhs_arg represents the base address, need the index separately.
                        // For simplicity in our simplified IR, we might just emit Assign.
                        // A more complete IR would be: STORE_ARRAY base, idx, rhs_arg
                        // But lhs_arg generated by compile_lvalue for Index might return a complex arg or we handle it specially.
                        // Here we keep it simple.
                        self.ir.emit(Op::Assign, rhs_arg, Arg::Empty, lhs_arg);
                    }
                    _ => {
                        self.ir.emit(Op::Assign, rhs_arg, Arg::Empty, lhs_arg);
                    }
                }
            }
            Statement::If(if_stmt) => {
                self.compile_if_stmt(if_stmt);
            }
            Statement::While(cond, body) => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                let (_, cond_arg) = self.compile_expression(cond);
                self.ir.emit(Op::JumpIfFalse, cond_arg, Arg::Empty, Arg::Label(end_label));
                
                self.loop_stack.push((start_label, end_label, None));
                self.compile_block(body);
                self.loop_stack.pop();
                
                self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(end_label));
            }
            Statement::For(var, iter, body) => {
                // Simplified for loop IR generation
                // Only supports x..y arrays
                let start_label = self.new_label();
                let end_label = self.new_label();
                
                self.loop_stack.push((start_label, end_label, None));
                self.compile_block(body);
                self.loop_stack.pop();
            }
            Statement::Loop(body) => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                
                self.loop_stack.push((start_label, end_label, None));
                self.compile_block(body);
                self.loop_stack.pop();
                
                self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(end_label));
            }
            Statement::Break(opt_e) => {
                if let Some(&(start_l, end_l, _)) = self.loop_stack.last() {
                    // Ignore expr for now
                    if let Some(e) = opt_e {
                        self.compile_expression(e);
                    }
                    self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(end_l));
                } else {
                    self.error("break 必须出现在循环体内".to_string());
                }
            }
            Statement::Continue => {
                if let Some(&(start_l, _, _)) = self.loop_stack.last() {
                    self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(start_l));
                } else {
                    self.error("continue 必须出现在循环体内".to_string());
                }
            }
        }
    }

    fn compile_if_stmt(&mut self, if_stmt: &IfStmt) {
        let (cond_ty, cond_arg) = self.compile_expression(&if_stmt.condition);
        let false_label = self.new_label();
        let end_label = self.new_label();

        self.ir.emit(Op::JumpIfFalse, cond_arg, Arg::Empty, Arg::Label(false_label));
        self.compile_block(&if_stmt.then_block);
        self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(end_label));
        self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(false_label));

        if let Some(else_part) = &if_stmt.else_part {
            match else_part {
                ElsePart::ElseBlock(b) => {
                    self.compile_block(b);
                }
                ElsePart::ElseIf(elif) => {
                    self.compile_if_stmt(elif);
                }
            }
        }

        self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(end_label));
    }

    fn compile_lvalue(&mut self, expr: &Expr) -> (Type, Arg) {
        match expr {
            Expr::Ident(name) => {
                if let Some(info) = self.lookup_variable(name) {
                    if !info.mutable {
                        self.error(format!("不可变变量 '{}' 不允许被二次赋值", name));
                    }
                    (info.ty.clone(), info.arg.clone())
                } else {
                    self.error(format!("左值变量 '{}' 未声明", name));
                    (Type::I32, Arg::Var(name.clone()))
                }
            }
            Expr::UnaryDeref(e) => {
                let (ty, arg) = self.compile_expression(e);
                let inner_ty = match ty {
                    Type::MutRef(t) => *t,
                    Type::Ref(_) => {
                        self.error("不可变引用不能被解引用赋值".to_string());
                        Type::I32
                    }
                    _ => {
                        self.error("只有引用类型可以被解引用".to_string());
                        Type::I32
                    }
                };
                (inner_ty, arg) // Arg is the pointer
            }
            Expr::Index(base, idx) => {
                let (base_ty, base_arg) = self.compile_lvalue(base);
                let (idx_ty, idx_arg) = self.compile_expression(idx);
                let elem_ty = if let Type::Array(t, _) = base_ty {
                    *t
                } else {
                    self.error("只有数组类型可以被索引".to_string());
                    Type::I32
                };
                let res = self.new_temp();
                // A complete IR would calculate offset. We just return a temp representing it for now.
                (elem_ty, res)
            }
            Expr::TupleIndex(base, idx) => {
                let (base_ty, base_arg) = self.compile_lvalue(base);
                let elem_ty = if let Type::Tuple(ts) = base_ty {
                    ts.get(*idx as usize).cloned().unwrap_or(Type::I32)
                } else {
                    self.error("只有元组类型可以被索引".to_string());
                    Type::I32
                };
                let res = self.new_temp();
                (elem_ty, res)
            }
            Expr::Paren(e) => self.compile_lvalue(e),
            _ => {
                self.error("非法的左值表达式".to_string());
                (Type::I32, Arg::Empty)
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expr) -> (Type, Arg) {
        match expr {
            Expr::IntLiteral(v) => (Type::I32, Arg::Int(*v)),
            Expr::Ident(name) => {
                if let Some(info) = self.lookup_variable(name) {
                    (info.ty.clone(), info.arg.clone())
                } else {
                    self.error(format!("变量 '{}' 未声明", name));
                    (Type::I32, Arg::Var(name.clone()))
                }
            }
            Expr::BinaryOp(l, op, r) => {
                let (lty, larg) = self.compile_expression(l);
                let (rty, rarg) = self.compile_expression(r);
                if lty != rty {
                    self.error(format!("二元运算类型不匹配: {:?} {} {:?}", lty, op, rty));
                }
                
                let res = self.new_temp();
                let ir_op = match op {
                    BinOp::Add => Op::Add,
                    BinOp::Sub => Op::Sub,
                    BinOp::Mul => Op::Mul,
                    BinOp::Div => Op::Div,
                    BinOp::Eq => Op::Eq,
                    BinOp::Ne => Op::Ne,
                    BinOp::Lt => Op::Lt,
                    BinOp::Le => Op::Le,
                    BinOp::Gt => Op::Gt,
                    BinOp::Ge => Op::Ge,
                };
                self.ir.emit(ir_op, larg, rarg, res.clone());
                (Type::I32, res) // Assuming all ops return i32 (booleans are represented as i32)
            }
            Expr::UnaryDeref(e) => {
                let (ty, arg) = self.compile_expression(e);
                let inner_ty = match ty {
                    Type::MutRef(t) | Type::Ref(t) => *t,
                    _ => {
                        self.error("只有引用类型可以被解引用".to_string());
                        Type::I32
                    }
                };
                let res = self.new_temp();
                self.ir.emit(Op::Deref, arg, Arg::Empty, res.clone());
                (inner_ty, res)
            }
            Expr::Ref(e) => {
                let (ty, arg) = self.compile_expression(e);
                let res = self.new_temp();
                self.ir.emit(Op::Ref, arg, Arg::Empty, res.clone());
                (Type::Ref(Box::new(ty)), res)
            }
            Expr::MutRef(e) => {
                let (ty, arg) = self.compile_expression(e);
                let res = self.new_temp();
                self.ir.emit(Op::Ref, arg, Arg::Empty, res.clone());
                (Type::MutRef(Box::new(ty)), res)
            }
            Expr::Call(name, args) => {
                // Not checking function signatures for simplicity
                for arg_expr in args {
                    let (_, arg_val) = self.compile_expression(arg_expr);
                    self.ir.emit(Op::Arg, arg_val, Arg::Empty, Arg::Empty);
                }
                let res = self.new_temp();
                self.ir.emit(Op::Call, Arg::Var(name.clone()), Arg::Int(args.len() as i64), res.clone());
                (Type::I32, res) // Defaulting all function calls to i32
            }
            Expr::Index(base, idx) => {
                let (base_ty, base_arg) = self.compile_expression(base);
                let (idx_ty, idx_arg) = self.compile_expression(idx);
                let elem_ty = if let Type::Array(t, _) = base_ty {
                    *t
                } else {
                    self.error("只有数组类型可以被索引".to_string());
                    Type::I32
                };
                let res = self.new_temp();
                self.ir.emit(Op::LoadArray, base_arg, idx_arg, res.clone());
                (elem_ty, res)
            }
            Expr::TupleIndex(base, idx) => {
                let (base_ty, base_arg) = self.compile_expression(base);
                let elem_ty = if let Type::Tuple(ts) = base_ty {
                    ts.get(*idx as usize).cloned().unwrap_or(Type::I32)
                } else {
                    self.error("只有元组类型可以被索引".to_string());
                    Type::I32
                };
                let res = self.new_temp();
                (elem_ty, res)
            }
            Expr::ArrayLiteral(elems) => {
                let mut ty = Type::I32;
                for (i, e) in elems.iter().enumerate() {
                    let (ety, _arg) = self.compile_expression(e);
                    if i == 0 { ty = ety; }
                }
                (Type::Array(Box::new(ty), elems.len() as i64), self.new_temp())
            }
            Expr::TupleLiteral(elems) => {
                let mut tys = Vec::new();
                for e in elems {
                    let (ety, _) = self.compile_expression(e);
                    tys.push(ety);
                }
                (Type::Tuple(tys), self.new_temp())
            }
            Expr::Paren(e) => self.compile_expression(e),
            Expr::ExprBlock(eb) => self.compile_expr_block(eb),
            Expr::IfExpr(cond, then_b, else_b) => {
                let (cond_ty, cond_arg) = self.compile_expression(cond);
                let false_label = self.new_label();
                let end_label = self.new_label();
                let res = self.new_temp();

                self.ir.emit(Op::JumpIfFalse, cond_arg, Arg::Empty, Arg::Label(false_label));
                let (then_ty, then_arg) = self.compile_expr_block(then_b);
                self.ir.emit(Op::Assign, then_arg, Arg::Empty, res.clone());
                self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(end_label));
                
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(false_label));
                let (else_ty, else_arg) = self.compile_expr_block(else_b);
                if then_ty != else_ty {
                    self.error(format!("If 表达式的分支类型不一致: {:?} vs {:?}", then_ty, else_ty));
                }
                self.ir.emit(Op::Assign, else_arg, Arg::Empty, res.clone());
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(end_label));

                (then_ty, res)
            }
            Expr::LoopExpr(body) => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                self.loop_stack.push((start_label, end_label, None));
                self.compile_block(body);
                self.loop_stack.pop();
                
                self.ir.emit(Op::Jump, Arg::Empty, Arg::Empty, Arg::Label(start_label));
                self.ir.emit(Op::Label, Arg::Empty, Arg::Empty, Arg::Label(end_label));
                
                (Type::I32, self.new_temp()) // Simplified loop expr return
            }
            _ => (Type::I32, Arg::Empty)
        }
    }
}
