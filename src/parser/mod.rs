use self::ast::{
    Opcode,
    Func,
    Assignment,
    Sign,
    Instruction as AstInstruction,
    Condition as AstCondition,
    IfBlock
};
use expressions::{
    ExpressionEvaluator,
    ExpressionMember,
    Operator,
    BinaryOperator,
    UnaryOperator,
    Variable,
};
use rules::{
    RulesEvaluator,
    Instruction,
};
use conditions::Condition;
use self::lexer::Tokenizer;

pub use self::ast::Expr;

mod ast;
mod lexer;
#[allow(dead_code)]
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
            Expr::Signed(sign, r) => {
                r.convert(res);
                match sign {
                    Sign::Plus => {}
                    Sign::Minus => res.push(ExpressionMember::Op(Operator::Unary(UnaryOperator::Minus))),
                }
            }
        }
    }
}

pub fn parse_rule(input: &str) -> Result<RulesEvaluator,String> {
    let tokenizer = Tokenizer::new(input);
    let tokenizer_mapped = tokenizer.map(|e| {
        e.map(|token| ((),token,()))
    });
    let instructions = match parser::parse_Rule(tokenizer_mapped) {
        Ok(t) => t,
        Err(e) => {
            return Err(format!("Parsing error {:?}", e));
        }
    };
    Ok(convert_instructions(instructions))
}

fn convert_instructions(ast: Vec<AstInstruction>) -> RulesEvaluator {
    let mut res = RulesEvaluator::new();
    for instruction in ast {
        match instruction {
            AstInstruction::Assignment(Assignment{local, variable, expr}) => {
                let i = Instruction::Assignment {
                    variable: Variable { local: local, name: variable },
                    expression: convert_expression(*expr),
                };
                res.push(i);
            }
            AstInstruction::If(IfBlock{condition, then_block, else_block}) => {
                let i = Instruction::IfBlock {
                    condition: convert_condition(*condition),
                    then_block: convert_instructions(then_block),
                    else_block: else_block.map(convert_instructions),
                };
                res.push(i);
            }
        }
    }
    res
}

fn convert_condition(ast: AstCondition) -> Condition {
    match ast {
        AstCondition::Logic(l, op, r) => {
            Condition::Logic(Box::new(convert_condition(*l)),
                             op,
                             Box::new(convert_condition(*r)))
        }
        AstCondition::Comparison(l, op, r) => {
            Condition::Comparison(convert_expression(*l), op, convert_expression(*r))
        }
        AstCondition::Exists(name) => {
            Condition::Exists(name)
        }
    }
}

fn convert_expression(expr: Expr) -> ExpressionEvaluator {
    let mut vec = Vec::new();
    expr.convert(&mut vec);
    ExpressionEvaluator::new(vec)
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

#[cfg(test)]
mod tests {
    use super::ast::Expr;
    use super::lexer::Tokenizer;
    use expressions::ExpressionEvaluator;

    fn parse_expr_to_ast(input: &str) -> Option<Box<Expr>> {
        let tokenizer = Tokenizer::new(input);
        let tokenizer_mapped = tokenizer.map(|e| {
            e.map(|token| ((),token,()))
        });
        super::parser::parse_Expr(tokenizer_mapped).ok()
    }

    fn parse_expr(input: &str) -> ExpressionEvaluator {
        let mut vec = vec![];
        let ast = parse_expr_to_ast(input).unwrap();
        ast.convert(&mut vec);
        ExpressionEvaluator::new(vec)
    }

    macro_rules! test_parse {
        ($to_parse:expr, $str:expr) => {
            let res = parse_expr_to_ast($to_parse).unwrap();
            assert_eq!(format!("{:?}", res),$str);
        }
    }

    #[test]
    fn simple_addition() {
        test_parse!("1+2", "(1 + 2)");
    }
    #[test]
    fn multiple_additions() {
        test_parse!("1 + 2 + 3", "((1 + 2) + 3)");
    }
    #[test]
    fn substraction() {
        test_parse!("1 - 2 + 3", "((1 - 2) + 3)");
        test_parse!("1 + 2 - 3", "((1 + 2) - 3)");
        test_parse!("1 - 2 - 3", "((1 - 2) - 3)");
    }
    #[test]
    fn priority() {
        test_parse!("1+2*3", "(1 + (2 * 3))");
        test_parse!("1*2+3", "((1 * 2) + 3)");
    }
    #[test]
    fn arity_minus() {
        test_parse!("1 -2", "(1 - 2)");
        test_parse!("- 1 -2", "(-(1) - 2)");
    }
    #[test]
    fn exponentiation_signed() {
        test_parse!("-2^2", "-((2 ^ 2))");
        assert!(parse_expr_to_ast("2^-2").is_none());
        test_parse!("2^(-2)", "(2 ^ -(2))");
    }
    #[test]
    fn exponentiation_recursivity() {
        test_parse!("2^3^4", "(2 ^ (3 ^ 4))");
    }
    #[test]
    fn parenthesis() {
        test_parse!("1 - (2 + 3)", "(1 - (2 + 3))");
    }
    #[test]
    fn local_global_variables() {
        let to_parse = "local";
        let res = parse_expr_to_ast(to_parse).unwrap();
        match *res {
            Expr::Variable{local: true, name} => {
                assert_eq!(&name, "local");
            }
            _ => panic!(),
        }
        let to_parse = "$global";
        let res = parse_expr_to_ast(to_parse).unwrap();
        match *res {
            Expr::Variable{local: false, name} => {
                assert_eq!(&name, "global");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_addition_variables() {
        test_parse!("local + $global * 3", "(local + ($global * 3))");
    }

    #[test]
    fn variable_names() {
        assert!(parse_expr_to_ast("test_underscore").is_some());
        assert!(parse_expr_to_ast("_bad_leading_underscore").is_none());
        assert!(parse_expr_to_ast("UpperCaseTest").is_some());
        assert!(parse_expr_to_ast("Point.Test").is_some());
    }

    // Test the evaluation
    #[test]
    fn evaluation() {
        let res = parse_expr("2^2^2").evaluate::<(),(),()>(&(), &()).unwrap();
        assert_eq!(res, 16.0);
        let res = parse_expr("-1-2-3").evaluate::<(),(),()>(&(), &()).unwrap();
        assert_eq!(res, -6.0);
    }
}
