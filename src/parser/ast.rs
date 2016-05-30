// Mostly taken from Nikomatsakis LALRPOP tutorial
use std::fmt::{Debug, Formatter, Error};

pub use conditions::{CompOp, LogicOp};

pub enum Instruction {
    Assignment(Assignment),
    If(IfBlock),
}

impl Instruction {
    pub fn new_assignment(l: bool, v: String, e: Box<Expr>) -> Instruction {
        Instruction::Assignment(Assignment::new(l,v,e))
    }

    pub fn new_if(c: Box<Condition>, t: Vec<Instruction>, e: Option<Vec<Instruction>>) -> Instruction {
        Instruction::If(IfBlock {
            condition: c,
            then_block: t,
            else_block: e
        })
    }
}

pub struct IfBlock {
    pub condition: Box<Condition>,
    pub then_block: Vec<Instruction>,
    pub else_block: Option<Vec<Instruction>>,
}

pub enum Condition {
    Comparison(Box<Expr>, CompOp, Box<Expr>),
    Logic(Box<Condition>, LogicOp, Box<Condition>),
    Exists(String),
}

pub struct Assignment {
    pub local: bool,
    pub variable: String,
    pub expr: Box<Expr>,
}

impl Assignment {
    pub fn new(local: bool, variable: String, expr: Box<Expr>) -> Assignment {
        Assignment {
            local: local,
            variable: variable,
            expr: expr,
        }
    }
}

pub enum Expr {
    Number(f64),
    Variable {
        local: bool,
        name: String,
    },
    Function(Func, Vec<Box<Expr>>),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Signed(Sign, Box<Expr>),
}

#[derive(Copy, Clone)]
pub enum Opcode {
    Plus,
    Minus,
    Multiply,
    Divide,
    Pow,
}

#[derive(Copy, Clone)]
pub enum Func {
    Rand,
    Min,
    Max,
    Sin,
    Cos,
}

#[derive(Copy,Clone)]
pub enum Sign {
    Plus,
    Minus,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expr::*;
        match *self {
            Number(n) => write!(fmt, "{:?}", n),
            Variable {local, ref name} => write!(fmt, "{}{}", if local {""} else {"$"}, name),
            Function(n, ref params) => {
                try!(write!(fmt, "{:?}(", n));
                let mut has_previous = false;
                for param in params {
                    if has_previous {
                        try!(write!(fmt, ", {:?}", param));
                    } else {
                        try!(write!(fmt, "{:?}", param));
                        has_previous = true;
                    }
                }
                write!(fmt, ")")
            }
            Op(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Signed(sign, ref e) => write!(fmt, "{:?}({:?})", sign, e),
        }
    }
}

impl Debug for Sign {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Sign::*;
        match *self {
            Minus => write!(fmt, "-"),
            Plus => write!(fmt, "+"),
        }
    }
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match *self {
            Multiply => write!(fmt, "*"),
            Divide => write!(fmt, "/"),
            Plus => write!(fmt, "+"),
            Minus => write!(fmt, "-"),
            Pow => write!(fmt, "^"),
        }
    }
}

impl Debug for Func {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Func::*;
        match *self {
            Rand => write!(fmt, "rand"),
            Min => write!(fmt, "min"),
            Max => write!(fmt, "max"),
            Sin => write!(fmt, "sin"),
            Cos => write!(fmt, "cos"),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Instruction::Assignment(ref a) => a.fmt(fmt),
            Instruction::If(ref i) => i.fmt(fmt),
        }
    }
}

impl Debug for Assignment {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let local = if self.local {""} else {"$"};
        write!(fmt, "{}{} = {:?};", local, self.variable, self.expr)
    }
}

impl Debug for IfBlock {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        try!(write!(fmt, "if {:?} {{ ", self.condition));
        for expr in self.then_block.iter() {
            try!(expr.fmt(fmt));
        }
        try!(write!(fmt, " }}"));
        if let Some(ref e) = self.else_block {
            try!(write!(fmt, " else {{ "));
            for expr in e.iter() {
                try!(expr.fmt(fmt));
            }
            try!(write!(fmt, " }}"));
        }
        Ok(())
    }
}

impl Debug for Condition {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Condition::Comparison(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Condition::Logic(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Condition::Exists(ref v) => write!(fmt, "exists({})", v),
        }
    }
}

impl Debug for CompOp {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::CompOp::*;
        let s = match *self {
            SuperiorStrict => ">",
            SuperiorEqual  => ">=",
            InferiorStrict => "<",
            InferiorEqual => "<=",
            Equal => "==",
            Different => "!=",
        };
        fmt.write_str(s)
    }
}

impl Debug for LogicOp {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::LogicOp::*;
        let s = match *self {
            And => "&&",
            Or  => "||",
        };
        fmt.write_str(s)
    }
}
