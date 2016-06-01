use std::collections::HashMap;
use std::mem;

use expressions::*;
use conditions::Condition;

pub trait Opaque: Sized {
    type Context;

    fn get(&mut self, context: &mut Self::Context) -> Option<StoreType<Self>>;
    fn set(&mut self,
           name: &str,
           value: StoreType<Self>,
           context: &mut Self::Context)
        -> Result<Option<StoreType<Self>>,()>;
}

impl Opaque for () {
    type Context = HashMap<String,StoreType<()>>;

    fn get(&mut self, _context: &mut Self::Context) -> Option<StoreType<()>> {
        None
    }
    fn set(&mut self,
           _name: &str,
           _value: StoreType<()>,
           _context: &mut Self::Context)
        -> Result<Option<StoreType<Self>>,()> {
            Err(())
        }
}

#[derive(Clone,Debug)]
pub enum Instruction {
    Assignment {
        variable: Variable,
        expression: ExpressionEvaluator,
    },
    IfBlock {
        condition: Condition,
        then_block: RulesEvaluator,
        else_block: Option<RulesEvaluator>,
    }
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
    pub fn evaluate<T: Opaque<Context=U> + Clone, U: Store<T>>(&self, global: &mut U) -> Result<(),RulesError> {
        let mut local = Scopes::new();
        self.evaluate_inner(global, &mut local)
    }

    pub fn new() -> RulesEvaluator {
        RulesEvaluator { instructions: Vec::new() }
    }

    fn evaluate_inner<T: Opaque<Context=U> + Clone, U: Store<T>>(&self, global: &mut U, local: &mut Scopes<T>) -> Result<(),RulesError> {
        // New scope
        local.push();
        for instruction in self.instructions.iter() {
            match *instruction {
                Instruction::Assignment {
                    variable: Variable { local: l, ref name },
                    ref expression,
                } => {
                    let res = try!(expression.evaluate(global, local));
                    if l {
                        local.set_variable(name, StoreType::F64(res));
                    } else {
                        let result = global.set_attribute(name, StoreType::F64(res));
                        if result.is_err() {
                            return Err(RulesError::CannotSetVariable(name.to_string()));
                        }
                    }
                }
                Instruction::IfBlock {
                    ref condition,
                    ref then_block,
                    ref else_block,
                } => {
                    if try!(condition.evaluate(global, local)) {
                        try!(then_block.evaluate_inner(global, local));
                    } else {
                        if let Some(ref e) = *else_block {
                            try!(e.evaluate_inner(global, local));
                        }
                    }
                }
            }
        }
        local.pop();
        Ok(())
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

struct Scopes<T> {
    inner: Vec<HashMap<String,StoreType<T>>>,
}

impl<T: Clone> Scopes<T> {
    fn push(&mut self) {
        self.inner.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.inner.pop();
    }

    fn new() -> Scopes<T> {
        Scopes { inner: Vec::with_capacity(4) }
    }

    fn set_variable(&mut self, name: &str, value: StoreType<T>) {
        // Will never return Err
        let _ = self.set_attribute(name, value);
    }
}

impl<T: Clone> Store<T> for Scopes<T> {
    fn get_attribute(&self, name: &str) -> Option<StoreType<T>> {
        for scope in self.inner.iter().rev() {
            let op = scope.get(name);
            if op.is_some() { return op.cloned(); }
        }
        None
    }

    fn set_attribute(&mut self, name: &str, value: StoreType<T>) -> Result<Option<StoreType<T>>,()> {
        for scope in self.inner.iter_mut().rev() {
            if let Some(ref mut e) = scope.get_mut(name) {
                return Ok(Some(mem::replace(e, value)));
            }
        }

        // The variable did not exist in any scope, create it
        let last_scope = self.inner.last_mut().unwrap();
        last_scope.insert(name.to_string(), value);
        Ok(None)
    }
}
