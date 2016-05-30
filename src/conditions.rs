use ordered_float::OrderedFloat;

use expressions::{ExpressionEvaluator,ExpressionError,Store};

use self::CompOp::*;
use self::LogicOp::*;
use self::Condition::*;

#[derive(Clone,Debug)]
pub enum Condition {
    Logic(Box<Condition>, LogicOp, Box<Condition>),
    Comparison(ExpressionEvaluator, CompOp, ExpressionEvaluator),
    Exists(String),
}

#[derive(Copy, Clone)]
pub enum CompOp {
    SuperiorStrict,
    SuperiorEqual,
    InferiorStrict,
    InferiorEqual,
    Equal,
    Different,
}

#[derive(Copy, Clone)]
pub enum LogicOp {
    And,
    Or,
}

impl Condition {
    pub fn evaluate<T: Store, V: Store>(&self, global: &T, local: &V)
                                        -> Result<bool,ExpressionError> {
        match *self {
            Logic(ref l, op, ref r) => {
                let result_left = try!(l.evaluate(global, local));
                match (result_left, op) {
                    (false, And) => Ok(false),
                    (true, Or) => Ok(true),
                    (false, Or) | (true, And) => r.evaluate(global, local),
                }                    
            }
            Comparison(ref left, op, ref right) => {
                let l = OrderedFloat(try!(left.evaluate(global, local)));
                let r = OrderedFloat(try!(right.evaluate(global, local)));
                match op {
                    SuperiorStrict => Ok(l > r),
                    InferiorStrict => Ok(l < r),
                    Equal => Ok(l == r),
                    Different => Ok(l != r),
                    SuperiorEqual => Ok(l >= r),
                    InferiorEqual => Ok(l <= r),
                }
            }
            Exists(ref name) => {
                Ok(global.get_attribute(name).is_some())
            }
        }
    }
}
