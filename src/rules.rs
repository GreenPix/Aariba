use std::collections::HashMap;

use expressions::*;

#[derive(Clone,Debug)]
pub struct RulesEvaluator {
    expressions: Vec<(Variable,ExpressionEvaluator)>,
}

#[derive(Clone,Debug)]
pub enum RulesError {
    Expression(ExpressionError),
    CannotSetVariable(String),
}

impl From<ExpressionError> for RulesError {
    fn from(err: ExpressionError) -> RulesError {
        RulesError::Expression(err)
    }
}

impl RulesEvaluator {
    pub fn evaluate<T: Store>(&self, global: &mut T) -> Result<(),RulesError> {
        let mut local_variables = HashMap::new();
        for &(Variable{local,ref name},ref expression) in self.expressions.iter() {
            let res = try!(expression.evaluate(global, &local_variables));
            if local {
                local_variables.insert(name.to_string(), res);
            } else {
                let result = global.set_attribute(name, res);
                if result.is_err() {
                    return Err(RulesError::CannotSetVariable(name.to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn new(expressions: Vec<(Variable,ExpressionEvaluator)>) -> RulesEvaluator {
        RulesEvaluator { expressions: expressions }
    }
}
