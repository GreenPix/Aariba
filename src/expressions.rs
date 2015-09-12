use std::ops::{Add,Sub,Mul,Div};
use std::collections::HashMap;

use self::ExpressionMember::*;
use self::ExpressionError::*;

pub trait Gettable {
    fn get_var(&self, var: &str) -> Option<ValueType>;
}

impl Gettable for HashMap<String,ValueType> {
    fn get_var(&self, var: &str) -> Option<ValueType> {
        self.get(var).cloned()
    }
}

impl Gettable for () {
    fn get_var(&self, _: &str) -> Option<ValueType> {
        None
    }
}

#[derive(Clone,Copy,Debug)]
pub enum ValueType {
    I64(i64),
    F32(f32),
}

impl Add for ValueType {
    type Output = ValueType;
    fn add(self, other: ValueType) -> ValueType {
        match (self, other) {
            (ValueType::I64(a), ValueType::I64(b)) => ValueType::I64(a+b),
            (ValueType::I64(a), ValueType::F32(b)) => ValueType::F32((a as f32)+b),
            (ValueType::F32(a), ValueType::I64(b)) => ValueType::F32(a+(b as f32)),
            (ValueType::F32(a), ValueType::F32(b)) => ValueType::F32(a+b),
        }
    }
}

impl Sub for ValueType {
    type Output = ValueType;
    fn sub(self, other: ValueType) -> ValueType {
        match (self, other) {
            (ValueType::I64(a), ValueType::I64(b)) => ValueType::I64(a-b),
            (ValueType::I64(a), ValueType::F32(b)) => ValueType::F32((a as f32)-b),
            (ValueType::F32(a), ValueType::I64(b)) => ValueType::F32(a-(b as f32)),
            (ValueType::F32(a), ValueType::F32(b)) => ValueType::F32(a-b),
        }
    }
}

impl Mul for ValueType {
    type Output = ValueType;
    fn mul(self, other: ValueType) -> ValueType {
        match (self, other) {
            (ValueType::I64(a), ValueType::I64(b)) => ValueType::I64(a*b),
            (ValueType::I64(a), ValueType::F32(b)) => ValueType::F32((a as f32)*b),
            (ValueType::F32(a), ValueType::I64(b)) => ValueType::F32(a*(b as f32)),
            (ValueType::F32(a), ValueType::F32(b)) => ValueType::F32(a*b),
        }
    }
}

impl Div for ValueType {
    type Output = ValueType;
    fn div(self, other: ValueType) -> ValueType {
        match (self, other) {
            (ValueType::I64(a), ValueType::I64(b)) => ValueType::I64(a/b),
            (ValueType::I64(a), ValueType::F32(b)) => ValueType::F32((a as f32)/b),
            (ValueType::F32(a), ValueType::I64(b)) => ValueType::F32(a/(b as f32)),
            (ValueType::F32(a), ValueType::F32(b)) => ValueType::F32(a/b),
        }
    }
}

impl ValueType {
    pub fn get_i64(self) -> Option<i64> {
        match self {
            ValueType::I64(a) => Some(a),
            _ => None,
        }
    }

    pub fn get_f32(self) -> Option<f32> {
        match self {
            ValueType::F32(a) => Some(a),
            _ => None,
        }
    }

    pub fn pow(self, rhs: ValueType) -> ValueType {
        match (self, rhs) {
            (ValueType::I64(a), ValueType::I64(b)) => {
                let b_as_u32;
                if b < 0  {
                    error!("Cannot pow to negative number {}", b);
                    b_as_u32 = 0;
                } else if b > (::std::u32::MAX as i64){
                    error!("Exponent out of range: {}", b);
                    b_as_u32 = ::std::u32::MAX;
                } else {
                    b_as_u32 = b as u32;
                }
                ValueType::I64(a.pow(b_as_u32))
            }
            (ValueType::I64(a), ValueType::F32(b)) => ValueType::F32((a as f32).powf(b)),
            (ValueType::F32(a), ValueType::I64(b)) => {
                let b_as_i32;
                if b < (::std::i32::MIN as i64)  {
                    error!("Exponent out of range: {}", b);
                    b_as_i32 = 0;
                } else if b > (::std::i32::MAX as i64){
                    error!("Exponent out of range: {}", b);
                    b_as_i32 = ::std::i32::MAX;
                } else {
                    b_as_i32 = b as i32;
                }
                ValueType::F32(a.powi(b_as_i32))
            }
            (ValueType::F32(a), ValueType::F32(b)) => ValueType::F32(a.powf(b)),
        }
    }
}

// Postfixed expression notation
// member1 member2 operator to do a conventional member1 operator member2
// A member can itself be an expression
//
// Few examples:
// 1 3 + 3 4 + *    => (1 + 3) * (3 + 4)
// 1 2 3 4 5 6 + * + * + => 1 + (2 * (3 + (4 * (5 + 6))))
#[derive(Clone,Debug)]
pub enum ExpressionMember {
    Op(Operator),
    Constant(ValueType),
    Variable {
        local: bool,
        name: String,
    },
}

#[derive(Clone,Copy,Debug)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Pow,
}

#[derive(Clone,Debug)]
pub struct ExpressionEvaluator {
    expression: Vec<ExpressionMember>,
}

#[derive(Debug,Clone)]
pub enum ExpressionError {
    VariableNotFound(String),
    InvalidExpression(String),
}

impl ExpressionEvaluator {
    /// Evaluates an expression using a context to get variables
    pub fn evaluate<T,V>(&self, global_variables: &T, local_variables: &V) -> Result<ValueType,ExpressionError>
    where T: Gettable,
          V: Gettable {
        let mut stack = Vec::new();
        for member in self.expression.iter() {
            match *member {
                Constant(value) => stack.push(value),
                Variable{local,ref name} => {
                    let value = if local {
                        try!(local_variables.get_var(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    } else {
                        try!(global_variables.get_var(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    };
                    stack.push(value);
                },
                Op(operator) => {
                    // First member will be the second one in the stack
                    let member2 = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", operator))));
                    let member1 = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", operator))));
                    let result = match operator {
                        Operator::Plus => member1 + member2,
                        Operator::Minus => member1 - member2,
                        Operator::Multiply => member1 * member2,
                        Operator::Divide => member1 / member2,
                        Operator::Pow => member1.pow(member2),
                    };
                    stack.push(result);
                }
            }
        }
        let result = try!(stack.pop().ok_or_else(|| InvalidExpression("No result at the end of the expression".to_string())));
        if !stack.is_empty() {
            return Err(InvalidExpression("Stack not empty at the end of the expression".to_string()));
        }
        Ok(result)
    }

    /// Get list of global variables referenced by this expression
    pub fn get_global_variable_list(&self) -> Vec<String> {
        self.expression.iter().filter_map(|member| {
            if let Variable{local: false, ref name} = *member {
                Some(name.clone())
            } else {
                None
            }
        }).collect()
    }

    /// Get list of local variables referenced by this expression
    pub fn get_local_variable_list(&self) -> Vec<String> {
        self.expression.iter().filter_map(|member| {
            if let Variable{local: true, ref name} = *member {
                Some(name.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn new(expression: Vec<ExpressionMember>) -> ExpressionEvaluator {
        ExpressionEvaluator {
            expression: expression
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::ExpressionMember::*;
    use super::Operator;
    use super::ValueType;
    use super::ExpressionEvaluator;
    #[test]
    fn evaluate_int() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(ValueType::I64(1)),
            Constant(ValueType::I64(2)),
            Op(Operator::Plus),
            ]);

        assert!(expression.evaluate(&context,&()).unwrap().get_i64().unwrap() == 3);
    }

    #[test]
    fn incorrect_expression() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(ValueType::I64(1)),
            Constant(ValueType::I64(2)),
            Op(Operator::Plus),
            Op(Operator::Multiply),
            ]);
        assert!(expression.evaluate(&context,&()).is_err());
    }

    #[test]
    fn evaluate_int_variable() {
        let mut context = HashMap::new();
        context.insert("forty_two".to_string(), ValueType::I64(42));
        context.insert("two".to_string(), ValueType::I64(2));
        // Calculates 2 * (forty_two / two) - 3
        let expression = ExpressionEvaluator::new(vec! [
            Constant(ValueType::I64(2)),
            Variable{local: false, name: "forty_two".to_string()},
            Variable{local: false, name: "two".to_string()},
            Op(Operator::Divide),
            Op(Operator::Multiply),
            Constant(ValueType::I64(3)),
            Op(Operator::Minus),
            ]);
        assert!(expression.evaluate(&context,&()).unwrap().get_i64().unwrap() == 39);
    }
}
