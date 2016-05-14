use self::ast::{
    Opcode,
    Func,
    Assignment,
};
use expressions::{
    ExpressionEvaluator,
    ExpressionMember,
    Operator,
    BinaryOperator,
    UnaryOperator,
    Variable,
};
use rules::RulesEvaluator;
use self::lexer::Tokenizer;

pub use self::ast::Expr;

mod ast;
mod lexer;
mod parser;

impl Expr {
    fn convert(self, res: &mut Vec<ExpressionMember>) {
        match self {
            Expr::Number(num) => {
                res.push(ExpressionMember::Constant(num));
            }
            Expr::Variable{local,name} => {
                res.push(ExpressionMember::Variable(Variable::new(local,name)));
            }
            Expr::Function(func, args) => {
                // TODO: insert check on function's number of operands
                for arg in args {
                    arg.convert(res);
                }
                let operator = func.into();
                res.push(operator);
            }
            Expr::Op(l, op, r) => {
                l.convert(res);
                r.convert(res);
                let operator = op.into();
                res.push(operator);
            }
        }
    }
}

pub fn parse_rule(input: &str) -> Result<RulesEvaluator,String> {
    let tokenizer = Tokenizer::new(input);
    let tokenizer_mapped = tokenizer.map(|e| {
        e.map(|token| ((),token,()))
    });
    let assignments = match parser::parse_Rule(tokenizer_mapped) {
        Ok(t) => t,
        Err(e) => {
            return Err(format!("Parsing error {:?}", e));
        }
    };
    let mut res = Vec::new();
    for Assignment{local, variable, expr} in assignments {
        let mut vec = Vec::new();
        expr.convert(&mut vec);
        res.push((Variable{local:local, name:variable}, ExpressionEvaluator::new(vec)));
    }
    Ok(RulesEvaluator::new(res))
}

impl Into<ExpressionMember> for Opcode {
    fn into(self) -> ExpressionMember {
        use self::ast::Opcode::*;
        match self {
            Plus => ExpressionMember::Op(Operator::Binary(BinaryOperator::Plus)),
            Minus => ExpressionMember::Op(Operator::Binary(BinaryOperator::Minus)),
            Multiply => ExpressionMember::Op(Operator::Binary(BinaryOperator::Multiply)),
            Divide => ExpressionMember::Op(Operator::Binary(BinaryOperator::Divide)),
            Pow => ExpressionMember::Op(Operator::Binary(BinaryOperator::Pow)),
        }
    }
}
impl Into<ExpressionMember> for Func {
    fn into(self) -> ExpressionMember {
        use self::ast::Func::*;
        match self {
            Sin => ExpressionMember::Op(Operator::Unary(UnaryOperator::Sin)),
            Cos => ExpressionMember::Op(Operator::Unary(UnaryOperator::Cos)),
            Min => ExpressionMember::Op(Operator::Binary(BinaryOperator::Min)),
            Max => ExpressionMember::Op(Operator::Binary(BinaryOperator::Max)),
            Rand => ExpressionMember::Op(Operator::Binary(BinaryOperator::Rand)),
        }
    }
}
