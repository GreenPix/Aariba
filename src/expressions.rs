use std::collections::HashMap;

use rand;

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
    Variable(Variable),
}

#[derive(Clone,Debug)]
pub struct Variable {
    pub local: bool,
    pub name: String,
}

impl Variable {
    pub fn new(local: bool, name: String) -> Variable {
        Variable {local: local, name: name}
    }
}

impl From<String> for Variable {
    fn from(mut name: String) -> Variable {
        let local;
        if name.starts_with("$") {
            name.remove(0);
            local = false;
        } else {
            local = true;
        }
        Variable {
            local: local,
            name: name,
        }
    }
}


#[derive(Clone,Copy,Debug)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}

impl Operator {
    fn apply(self, stack: &mut Vec<f64>) -> Result<f64,ExpressionError> {
        match self {
            Operator::Unary(op) => {
                let operand = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", self))));
                Ok(op.apply(operand))
            }
            Operator::Binary(op) => {
                let rhs = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", self))));
                let lhs = try!(stack.pop().ok_or_else(|| InvalidExpression(format!("Missing member for operator {:?}", self))));
                Ok(op.apply(lhs,rhs))
            },
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Pow,
    Min,
    Max,
    Rand,
}

impl BinaryOperator {
    fn apply(self, lhs: f64, rhs: f64) -> f64 {
        match self {
            BinaryOperator::Plus => lhs + rhs,
            BinaryOperator::Minus => lhs - rhs,
            BinaryOperator::Multiply => lhs * rhs,
            BinaryOperator::Divide => lhs / rhs,
            BinaryOperator::Pow => lhs.powf(rhs),
            BinaryOperator::Min => if lhs < rhs {lhs} else {rhs},
            BinaryOperator::Max => if lhs > rhs {lhs} else {rhs},
            BinaryOperator::Rand => {
                let (min,max) = if lhs < rhs {(lhs,rhs)} else {(rhs,lhs)};
                let rand: f64 = rand::random();
                min + rand * (max - min)
            }
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub enum UnaryOperator {
    Sin,
    Cos,
}

impl UnaryOperator {
    fn apply(self, operand: f64) -> f64 {
        match self {
            UnaryOperator::Sin => { operand.sin() }
            UnaryOperator::Cos => { operand.cos() }
        }
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
                ExpressionMember::Constant(value) => stack.push(value),
                ExpressionMember::Variable(Variable{local,ref name}) => {
                    let value = if local {
                        // Error to reference an undefined variable
                        try!(local_variables.get_attribute(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    } else {
                        try!(global_variables.get_attribute(&name).ok_or_else(|| VariableNotFound(name.clone())))
                    };
                    stack.push(value);
                },
                ExpressionMember::Op(operator) => {
                    let result = try!(operator.apply(&mut stack));
                    stack.push(result);
                    // First member will be the second one in the stack
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
            if let ExpressionMember::Variable(Variable{local: false, ref name}) = *member {
                Some(name.clone())
            } else {
                None
            }
        }).collect()
    }

    /// Get list of local variables referenced by this expression
    pub fn get_local_variable_list(&self) -> Vec<String> {
        self.expression.iter().filter_map(|member| {
            if let ExpressionMember::Variable(Variable{local: true, ref name}) = *member {
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
    use super::BinaryOperator;
    use super::ExpressionEvaluator;
    #[test]
    fn evaluate_int() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(1.0),
            Constant(2.0),
            Op(Operator::Binary(BinaryOperator::Plus)),
            ]);

        assert!(expression.evaluate(&context,&()).unwrap() == 3.0);
    }

    #[test]
    fn incorrect_expression() {
        let context = HashMap::new();
        let expression = ExpressionEvaluator::new(vec! [
            Constant(1.0),
            Constant(2.0),
            Op(Operator::Binary(BinaryOperator::Plus)),
            Op(Operator::Binary(BinaryOperator::Multiply)),
            ]);
        assert!(expression.evaluate(&context,&()).is_err());
    }

    #[test]
    fn evaluate_int_variable() {
        use super::Variable as Var;
        let mut context = HashMap::new();
        context.insert("forty_two".to_string(), 42.0);
        context.insert("two".to_string(), 2.0);
        // Calculates 2 * (forty_two / two) - 3
        let expression = ExpressionEvaluator::new(vec! [
            Constant(2.0),
            Variable(Var::new(false, "forty_two".to_string())),
            Variable(Var::new(false, "two".to_string())),
            Op(Operator::Binary(BinaryOperator::Divide)),
            Op(Operator::Binary(BinaryOperator::Multiply)),
            Constant(3.0),
            Op(Operator::Binary(BinaryOperator::Minus)),
            ]);
        assert!(expression.evaluate(&context,&()).unwrap() == 39.0);
    }
}
