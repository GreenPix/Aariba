use std::collections::HashMap;

use self::ExpressionMember::*;
use self::ExpressionError::*;

pub trait Store {
    fn get_attribute(&self, var: &str) -> Option<f64>;
    /// Set the attribute "var" to "value"
    ///
    /// Returns the old value, if any
    fn set_attribute(&mut self, var: &str, value: f64) -> Result<Option<f64>,()>;
}

impl Store for HashMap<String,f64> {
    fn get_attribute(&self, var: &str) -> Option<f64> {
        self.get(var).cloned()
    }

    fn set_attribute(&mut self, var: &str, value: f64) -> Result<Option<f64>,()> {
        Ok(self.insert(var.into(), value))
    }
}

impl Store for () {
    fn get_attribute(&self, _: &str) -> Option<f64> {
        None
    }

    fn set_attribute(&mut self, _: &str, _: f64) -> Result<Option<f64>,()> {
        Err(())
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
    Constant(f64),
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

impl Operator {
    fn apply(self, lhs: f64, rhs: f64) -> Result<f64,ExpressionError> {
        let result = match self {
            Operator::Plus => lhs + rhs,
            Operator::Minus => lhs - rhs,
            Operator::Multiply => lhs * rhs,
            Operator::Divide => lhs / rhs,
            Operator::Pow => lhs.powf(rhs),
        };
        Ok(result)
    }
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
    pub fn evaluate<T,V>(&self, global_variables: &T, local_variables: &V) -> Result<f64,ExpressionError>
    where T: Store,
          V: Store {
        // The algorithm to execute such an expression is fairly simple:
        //  - Create a stack to hold temporary values
        //  - Iterate through the expression members
        //   * If it is a number / variable, push it on the stack
        //   * If it is an operator, pop the correct number of elements from the stack, compute the
        //   result and push it on the stack
        //  - At the end of the expression, the stack must contain one single value, which is the
        //  result
        let mut stack = Vec::new();
        for member in self.expression.iter() {
            match *member {
                Constant(value) => stack.push(value),
                Variable{local,ref name} => {
                    let value = if local {
                        // Error to reference an undefined variable
                        try!(local_variables.get_attribute(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    } else {
                        try!(global_variables.get_attribute(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    };
                    stack.push(value);
                },
                Op(operator) => {
                    // First member will be the second one in the stack
                    let member2 = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", operator))));
                    let member1 = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", operator))));
                    let result = try!(operator.apply(member1,member2));
                    stack.push(result);
                }
            }
        }
        let result = try!(stack.pop().ok_or_else(|| InvalidExpression("No result at the end of the expression".into())));
        if !stack.is_empty() {
            return Err(InvalidExpression("Stack not empty at the end of the expression".into()));
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
    use super::ExpressionEvaluator;
    #[test]
    fn evaluate_int() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(1.0),
            Constant(2.0),
            Op(Operator::Plus),
            ]);

        assert!(expression.evaluate(&context,&()).unwrap() == 3.0);
    }

    #[test]
    fn incorrect_expression() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(1.0),
            Constant(2.0),
            Op(Operator::Plus),
            Op(Operator::Multiply),
            ]);
        assert!(expression.evaluate(&context,&()).is_err());
    }

    #[test]
    fn evaluate_int_variable() {
        let mut context = HashMap::new();
        context.insert("forty_two".to_string(), 42.0);
        context.insert("two".to_string(), 2.0);
        // Calculates 2 * (forty_two / two) - 3
        let expression = ExpressionEvaluator::new(vec! [
            Constant(2.0),
            Variable{local: false, name: "forty_two".to_string()},
            Variable{local: false, name: "two".to_string()},
            Op(Operator::Divide),
            Op(Operator::Multiply),
            Constant(3.0),
            Op(Operator::Minus),
            ]);
        assert!(expression.evaluate(&context,&()).unwrap() == 39.0);
    }
}
