use std::collections::HashMap;

use expressions::*;

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

#[derive(Clone,Debug)]
pub enum Instruction {
    Assignment {
        variable: Variable,
        expression: ExpressionEvaluator,
    },
    If {
        condition: Variable,
        then: RulesEvaluator,
        else_branch: Option<RulesEvaluator>,
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
                Instruction::If {ref condition,ref then,ref else_branch} => {
                    let res = if condition.local {
                        local_variables.get_attribute(&condition.name).is_some()
                    } else {
                        global.get_attribute(&condition.name).is_some()
                    };
                    if res {
                        try!(then.evaluate(global));
                    } else if let Some(ref else_br) = *else_branch {
                        try!(else_br.evaluate(global));
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

#[cfg(test)]
mod tests {
    use parser;
    use std::collections::HashMap;

    #[test]
    fn conditions() {
        let rule1_to_parse = "if $a { $b=0; } else { $b=1; }";
        let rule1 = parser::rules_evaluator(rule1_to_parse).unwrap();
        let mut global = HashMap::new();
        rule1.evaluate(&mut global).unwrap();
        let rule2_to_parse = r"
        $c=0;
        if $c { $d=0; } else { $d=1; }";
        let rule2 = parser::rules_evaluator(rule2_to_parse).unwrap();
        rule2.evaluate(&mut global).unwrap();
        assert_eq!(*global.get("b").unwrap(), 1.0);
        assert_eq!(*global.get("d").unwrap(), 0.0);
    }
}
