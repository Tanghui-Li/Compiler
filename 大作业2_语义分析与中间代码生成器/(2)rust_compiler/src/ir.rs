use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
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
    Assign,
    Jump,
    JumpIfFalse,
    Label,
    Param,
    Call,
    Return,
    Arg,
    // Extensions for arrays/pointers if needed
    Ref,
    Deref,
    DerefAssign,
    LoadArray,
    StoreArray,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "ADD"),
            Op::Sub => write!(f, "SUB"),
            Op::Mul => write!(f, "MUL"),
            Op::Div => write!(f, "DIV"),
            Op::Eq => write!(f, "EQ"),
            Op::Ne => write!(f, "NE"),
            Op::Lt => write!(f, "LT"),
            Op::Le => write!(f, "LE"),
            Op::Gt => write!(f, "GT"),
            Op::Ge => write!(f, "GE"),
            Op::Assign => write!(f, "ASSIGN"),
            Op::Jump => write!(f, "JUMP"),
            Op::JumpIfFalse => write!(f, "JUMP_IF_FALSE"),
            Op::Label => write!(f, "LABEL"),
            Op::Param => write!(f, "PARAM"),
            Op::Call => write!(f, "CALL"),
            Op::Return => write!(f, "RETURN"),
            Op::Arg => write!(f, "ARG"),
            Op::Ref => write!(f, "REF"),
            Op::Deref => write!(f, "DEREF"),
            Op::DerefAssign => write!(f, "DEREF_ASSIGN"),
            Op::LoadArray => write!(f, "LOAD_ARRAY"),
            Op::StoreArray => write!(f, "STORE_ARRAY"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Empty,
    Int(i64),
    Var(String),
    Temp(usize),
    Label(usize),
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Empty => write!(f, "-"),
            Arg::Int(v) => write!(f, "{}", v),
            Arg::Var(n) => write!(f, "{}", n),
            Arg::Temp(t) => write!(f, "t{}", t),
            Arg::Label(l) => write!(f, "L{}", l),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quadruple {
    pub op: Op,
    pub arg1: Arg,
    pub arg2: Arg,
    pub result: Arg,
}

impl fmt::Display for Quadruple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:^10}, {:^10}, {:^10}, {:^10})", 
               self.op.to_string(), 
               self.arg1.to_string(), 
               self.arg2.to_string(), 
               self.result.to_string())
    }
}

#[derive(Debug)]
pub struct IRProgram {
    pub instructions: Vec<Quadruple>,
}

impl IRProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, quad: Quadruple) -> usize {
        let index = self.instructions.len();
        self.instructions.push(quad);
        index
    }

    pub fn emit(&mut self, op: Op, arg1: Arg, arg2: Arg, result: Arg) -> usize {
        self.push(Quadruple { op, arg1, arg2, result })
    }

    pub fn print(&self) {
        for (i, quad) in self.instructions.iter().enumerate() {
            if quad.op == Op::Label {
                println!("{}:", quad.result);
            } else {
                println!("{:4}: {}", i, quad);
            }
        }
    }
}
