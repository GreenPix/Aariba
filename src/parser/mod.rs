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

#[cfg(test)]
mod tests {
    use super::ast::Expr;
    use super::lexer::Tokenizer;

    fn parse_expr(input: &str) -> Option<Box<Expr>> {
        let tokenizer = Tokenizer::new(input);
        let tokenizer_mapped = tokenizer.map(|e| {
            e.map(|token| ((),token,()))
        });
        super::parser::parse_Expr(tokenizer_mapped).ok()
    }
    macro_rules! test_parse {
        ($to_parse:expr, $str:expr) => {
            let res = parse_expr($to_parse).unwrap();
            assert_eq!(format!("{:?}", res),$str);
        }
    }

    // These tests should ideally parse correctly
    // The current lexer/parser cannot differenciate between a unary or binary '-'
    // So the current rule is that if '-' is immediatly followed by a number, it is unary
    // If not it is binary
    #[test]
    fn weird_behaviour() {
        // Here '-' is treated as unary whereas it should be binary
        assert!(parse_expr("1 -2").is_none());
        // Here '-' is treated as binary whereas it should be unnary
        assert!(parse_expr("- 2").is_none());
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
    fn local_global_variables() {
        let to_parse = "local";
        let res = parse_expr(to_parse).unwrap();
        match *res {
            Expr::Variable{local: true, name} => {
                assert_eq!(&name, "local");
            }
            _ => panic!(),
        }
        let to_parse = "$global";
        let res = parse_expr(to_parse).unwrap();
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
}
