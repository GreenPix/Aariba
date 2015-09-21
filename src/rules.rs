use std::collections::HashMap;

use expressions::*;

#[derive(Clone,Debug)]
pub struct Variable {
    local: bool,
    name: String,
}

impl Variable {
    pub fn new(local: bool, name: String) -> Variable {
        Variable {local: local, name: name}
    }
}

#[derive(Clone,Debug)]
pub enum Instruction {
    Assignment {
        variable: Variable,
        expression: ExpressionEvaluator,
    },
}

#[derive(Clone,Debug)]
pub struct RulesEvaluator {
    instructions: Vec<Instruction>,
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
        for instruction in self.instructions.iter() {
            match *instruction {
                Instruction::Assignment {variable: Variable{local,ref name},ref expression} => {
                    let res = try!(expression.evaluate(global, &local_variables));
                    if local {
                        local_variables.insert(name.clone(), res);
                    } else {
                        let result = global.set_attribute(name, res);
                        if result.is_err() {
                            return Err(RulesError::CannotSetVariable(name.clone()));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn new(instructions: Vec<Instruction>) -> RulesEvaluator {
        RulesEvaluator { instructions: instructions }
    }
}
