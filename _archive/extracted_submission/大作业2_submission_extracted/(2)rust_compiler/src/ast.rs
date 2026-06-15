/// Abstract Syntax Tree definitions for the Rust-like language
use std::fmt;

/// Top-level program: a list of declarations
#[derive(Debug)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// A declaration is a function declaration
#[derive(Debug)]
pub enum Declaration {
    Function(FunctionDecl),
}

/// Function declaration
#[derive(Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    #[allow(dead_code)]
    pub is_expr_body: bool, // rule 7.2: function expression block as body
}

/// Function parameter
#[derive(Debug)]
pub struct Param {
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
}

/// Type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    Ref(Box<Type>),        // &T
    MutRef(Box<Type>),     // &mut T
    Array(Box<Type>, i64), // [T; N]
    Tuple(Vec<Type>),      // (T1, T2, ...)
}

/// Block: { stmts }
#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

/// Expression block that can end with an expression (rule 7.0)
#[derive(Debug)]
pub struct ExprBlock {
    pub stmts: Vec<Statement>,
    pub tail_expr: Option<Box<Expr>>,
}

/// Statement
#[derive(Debug)]
pub enum Statement {
    Empty,                                  // ;
    ExprStmt(Expr),                         // expr ;
    Return(Option<Expr>),                   // return ; | return expr ;
    VarDecl(VarDecl),                       // let ... ;
    Assign(Expr, Expr),                     // lvalue = expr ;
    VarDeclAssign(VarDecl, Expr),           // let ... = expr ;
    If(IfStmt),                             // if expr block else_part
    While(Expr, Block),                     // while expr block
    For(VarDeclName, Expr, Block),          // for var in iterable block
    Loop(Block),                            // loop block
    Break(Option<Expr>),                    // break ; | break expr ;
    Continue,                               // continue ;
}

/// Variable declaration name part (without init)
#[derive(Debug)]
pub struct VarDecl {
    pub mutable: bool,
    pub name: String,
    #[allow(dead_code)]
    pub ty: Option<Type>,
}

/// Just the name portion for for-loops
#[derive(Debug)]
pub struct VarDeclName {
    pub mutable: bool,
    pub name: String,
    #[allow(dead_code)]
    pub ty: Option<Type>,
}

/// If statement
#[derive(Debug)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_part: Option<ElsePart>,
}

/// Else branch
#[derive(Debug)]
pub enum ElsePart {
    ElseBlock(Block),
    ElseIf(Box<IfStmt>),
}

/// Expressions
#[derive(Debug)]
pub enum Expr {
    IntLiteral(i64),
    Ident(String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryDeref(Box<Expr>),        // *expr  (rule 6.4)
    Ref(Box<Expr>),               // &expr  (rule 6.2)
    MutRef(Box<Expr>),            // &mut expr  (rule 6.3)
    Call(String, Vec<Expr>),      // fn(args)  (rule 3.5)
    Index(Box<Expr>, Box<Expr>),  // expr[expr]  (rule 8.3)
    TupleIndex(Box<Expr>, i64),   // expr.N  (rule 9.3)
    ArrayLiteral(Vec<Expr>),      // [a, b, c]  (rule 8.2)
    TupleLiteral(Vec<Expr>),      // (a, b)  (rule 9.2)
    Range(Box<Expr>, Box<Expr>),  // a..b  (rule 5.2 iterable)
    ExprBlock(ExprBlock),         // { stmts; expr }  (rule 7.0/7.1)
    IfExpr(Box<Expr>, Box<ExprBlock>, Box<ExprBlock>), // if cond {e} else {e} (rule 7.3)
    LoopExpr(Block),              // loop { ... }  (rule 7.4)
    Paren(Box<Expr>),             // (expr)
}

/// Binary operators
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::Ge => write!(f, ">="),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::Ref(t) => write!(f, "&{}", t),
            Type::MutRef(t) => write!(f, "&mut {}", t),
            Type::Array(t, n) => write!(f, "[{}; {}]", t, n),
            Type::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}

// Pretty printer for AST
pub fn print_ast(program: &Program, indent: usize) {
    println!("{}Program", pad(indent));
    for decl in &program.declarations {
        print_decl(decl, indent + 2);
    }
}

fn pad(n: usize) -> String {
    " ".repeat(n)
}

fn print_decl(decl: &Declaration, indent: usize) {
    match decl {
        Declaration::Function(f) => {
            let ret = match &f.return_type {
                Some(t) => format!(" -> {}", t),
                None => String::new(),
            };
            println!("{}FunctionDecl: {}({}){}", pad(indent), f.name,
                f.params.iter().map(|p| {
                    let m = if p.mutable { "mut " } else { "" };
                    format!("{}{}: {}", m, p.name, p.ty)
                }).collect::<Vec<_>>().join(", "),
                ret
            );
            print_block(&f.body, indent + 2);
        }
    }
}

fn print_block(block: &Block, indent: usize) {
    println!("{}Block", pad(indent));
    for stmt in &block.stmts {
        print_stmt(stmt, indent + 2);
    }
}

fn print_stmt(stmt: &Statement, indent: usize) {
    match stmt {
        Statement::Empty => println!("{}EmptyStmt", pad(indent)),
        Statement::ExprStmt(e) => {
            println!("{}ExprStmt", pad(indent));
            print_expr(e, indent + 2);
        }
        Statement::Return(None) => println!("{}Return (void)", pad(indent)),
        Statement::Return(Some(e)) => {
            println!("{}Return", pad(indent));
            print_expr(e, indent + 2);
        }
        Statement::VarDecl(v) => {
            let m = if v.mutable { "mut " } else { "" };
            let t = match &v.ty {
                Some(ty) => format!(": {}", ty),
                None => String::new(),
            };
            println!("{}VarDecl: let {}{}{}", pad(indent), m, v.name, t);
        }
        Statement::Assign(lhs, rhs) => {
            println!("{}Assign", pad(indent));
            print_expr(lhs, indent + 2);
            print_expr(rhs, indent + 2);
        }
        Statement::VarDeclAssign(v, e) => {
            let m = if v.mutable { "mut " } else { "" };
            let t = match &v.ty {
                Some(ty) => format!(": {}", ty),
                None => String::new(),
            };
            println!("{}VarDeclAssign: let {}{}{} = ...", pad(indent), m, v.name, t);
            print_expr(e, indent + 2);
        }
        Statement::If(if_stmt) => print_if(if_stmt, indent),
        Statement::While(cond, body) => {
            println!("{}While", pad(indent));
            print_expr(cond, indent + 2);
            print_block(body, indent + 2);
        }
        Statement::For(var, iter_expr, body) => {
            let m = if var.mutable { "mut " } else { "" };
            println!("{}For: {}{}", pad(indent), m, var.name);
            print_expr(iter_expr, indent + 2);
            print_block(body, indent + 2);
        }
        Statement::Loop(body) => {
            println!("{}Loop", pad(indent));
            print_block(body, indent + 2);
        }
        Statement::Break(None) => println!("{}Break", pad(indent)),
        Statement::Break(Some(e)) => {
            println!("{}Break (with value)", pad(indent));
            print_expr(e, indent + 2);
        }
        Statement::Continue => println!("{}Continue", pad(indent)),
    }
}

fn print_if(if_stmt: &IfStmt, indent: usize) {
    println!("{}If", pad(indent));
    print_expr(&if_stmt.condition, indent + 2);
    print_block(&if_stmt.then_block, indent + 2);
    match &if_stmt.else_part {
        None => {}
        Some(ElsePart::ElseBlock(b)) => {
            println!("{}Else", pad(indent));
            print_block(b, indent + 2);
        }
        Some(ElsePart::ElseIf(elif)) => {
            println!("{}ElseIf", pad(indent));
            print_if(elif, indent + 2);
        }
    }
}

fn print_expr(expr: &Expr, indent: usize) {
    match expr {
        Expr::IntLiteral(n) => println!("{}Int({})", pad(indent), n),
        Expr::Ident(s) => println!("{}Ident({})", pad(indent), s),
        Expr::BinaryOp(l, op, r) => {
            println!("{}BinOp({})", pad(indent), op);
            print_expr(l, indent + 2);
            print_expr(r, indent + 2);
        }
        Expr::UnaryDeref(e) => {
            println!("{}Deref(*)", pad(indent));
            print_expr(e, indent + 2);
        }
        Expr::Ref(e) => {
            println!("{}Ref(&)", pad(indent));
            print_expr(e, indent + 2);
        }
        Expr::MutRef(e) => {
            println!("{}MutRef(&mut)", pad(indent));
            print_expr(e, indent + 2);
        }
        Expr::Call(name, args) => {
            println!("{}Call: {}({} args)", pad(indent), name, args.len());
            for a in args {
                print_expr(a, indent + 2);
            }
        }
        Expr::Index(base, idx) => {
            println!("{}Index", pad(indent));
            print_expr(base, indent + 2);
            print_expr(idx, indent + 2);
        }
        Expr::TupleIndex(base, idx) => {
            println!("{}TupleIndex(.{})", pad(indent), idx);
            print_expr(base, indent + 2);
        }
        Expr::ArrayLiteral(elems) => {
            println!("{}ArrayLiteral({} elems)", pad(indent), elems.len());
            for e in elems {
                print_expr(e, indent + 2);
            }
        }
        Expr::TupleLiteral(elems) => {
            println!("{}TupleLiteral({} elems)", pad(indent), elems.len());
            for e in elems {
                print_expr(e, indent + 2);
            }
        }
        Expr::Range(lo, hi) => {
            println!("{}Range(..)", pad(indent));
            print_expr(lo, indent + 2);
            print_expr(hi, indent + 2);
        }
        Expr::ExprBlock(eb) => {
            println!("{}ExprBlock", pad(indent));
            for s in &eb.stmts {
                print_stmt(s, indent + 2);
            }
            if let Some(te) = &eb.tail_expr {
                println!("{}TailExpr:", pad(indent + 2));
                print_expr(te, indent + 4);
            }
        }
        Expr::IfExpr(cond, then_eb, else_eb) => {
            println!("{}IfExpr", pad(indent));
            print_expr(cond, indent + 2);
            println!("{}Then:", pad(indent + 2));
            if let Some(te) = &then_eb.tail_expr {
                print_expr(te, indent + 4);
            }
            println!("{}Else:", pad(indent + 2));
            if let Some(te) = &else_eb.tail_expr {
                print_expr(te, indent + 4);
            }
        }
        Expr::LoopExpr(body) => {
            println!("{}LoopExpr", pad(indent));
            print_block(body, indent + 2);
        }
        Expr::Paren(e) => {
            println!("{}Paren", pad(indent));
            print_expr(e, indent + 2);
        }
    }
}
