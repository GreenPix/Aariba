// Mostly taken from Nikomatsakis LALRPOP tutorial
use std::fmt::{Debug, Formatter, Error};

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
