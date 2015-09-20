use expressions::{
    ExpressionEvaluator,
    ExpressionMember,
    Operator,
    BinaryOperator,
    UnaryOperator,
};

pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Pow,
}

pub fn combine(left: Node, right_vec: Vec<(BinOp,Node)>) -> Node {
    let mut res = left;
    for (op, right) in right_vec {
        match op {
            BinOp::Plus => {
                res = Node::Plus((Box::new(res),Box::new(right)));
            }
            BinOp::Minus => {
                res = Node::Minus((Box::new(res),Box::new(right)));
            }
            BinOp::Multiply => {
                res = Node::Multiply((Box::new(res),Box::new(right)));
            }
            BinOp::Divide => {
                res = Node::Divide((Box::new(res),Box::new(right)));
            }
            BinOp::Pow => {
                res = Node::Pow((Box::new(res),Box::new(right)));
            }
        }
    }
    res
}

#[derive(Debug,Clone,Copy)]
pub enum FunctionKind {
    Sin,
    Cos,
    Rand,
    Min,
    Max,
}

impl Into<ExpressionMember> for FunctionKind {
    fn into(self) -> ExpressionMember {
        match self {
            FunctionKind::Sin => {ExpressionMember::Op(Operator::Unary(UnaryOperator::Sin))}
            FunctionKind::Cos => {ExpressionMember::Op(Operator::Unary(UnaryOperator::Cos))}
            FunctionKind::Min => {ExpressionMember::Op(Operator::Binary(BinaryOperator::Min))}
            FunctionKind::Max => {ExpressionMember::Op(Operator::Binary(BinaryOperator::Max))}
            FunctionKind::Rand => {ExpressionMember::Op(Operator::Binary(BinaryOperator::Rand))}
        }
    }
}

#[derive(Debug,Clone)]
pub enum Node {
    Plus((Box<Node>,Box<Node>)),
    Minus((Box<Node>,Box<Node>)),
    Multiply((Box<Node>,Box<Node>)),
    Divide((Box<Node>,Box<Node>)),
    Pow((Box<Node>,Box<Node>)),
    Function {
        function: FunctionKind,
        operands: Vec<Node>,
    },
    F64(f64),
    Variable {
        local: bool,
        name: String,
    },
}

pub fn convert(expression: Node) -> ExpressionEvaluator {
    let mut res = Vec::new();
    expression.convert(&mut res);
    
    ExpressionEvaluator::new(res)
}

impl Node {
    fn convert(self, res: &mut Vec<ExpressionMember>) {
        match self {
            Node::Plus((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Binary(BinaryOperator::Plus)));
            }
            Node::Minus((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Binary(BinaryOperator::Minus)));
            }
            Node::Multiply((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Binary(BinaryOperator::Multiply)));
            }
            Node::Divide((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Binary(BinaryOperator::Divide)));
            }
            Node::Pow((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Binary(BinaryOperator::Pow)));
            }
            Node::F64(num) => {
                res.push(ExpressionMember::Constant(num));
            }
            Node::Variable{local,name} => {
                res.push(ExpressionMember::Variable{local:local,name:name});
            }
            Node::Function{function,operands} => {
                // TODO: insert check on function's number of operands
                for operand in operands {
                    operand.convert(res);
                }
                let operator = function.into();
                res.push(operator);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Node};
    use super::super::parser;
    #[test]
    fn simple_addition() {
        let to_parse = "1 + 2";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((box Node::F64(1.0), box Node::F64(2.0))) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn multiple_addition() {
        let to_parse = "1 + 2 + 3";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Plus((box Node::F64(1.0), box Node::F64(2.0))),
                    box Node::F64(3.0)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn substraction_associativity() {
        let to_parse = "1 - 2 + 3";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Minus((box Node::F64(1.0), box Node::F64(2.0))),
                    box Node::F64(3.0)
                    )) => {}
            _ => panic!(),
        }
        let to_parse = "1 + 2 - 3";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Minus((
                    box Node::Plus((box Node::F64(1.0), box Node::F64(2.0))),
                    box Node::F64(3.0)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn priority() {
        let to_parse = "1 + 2 * 3";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::F64(1.0),
                    box Node::Multiply((box Node::F64(2.0), box Node::F64(3.0)))
                    )) => {}
            _ => panic!(),
        }
        let to_parse = "1 * 2 + 3";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Multiply((box Node::F64(1.0), box Node::F64(2.0))),
                    box Node::F64(3.0)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn local_global_variables() {
        let to_parse = "local + $global";
        let res = parser::expression_tree(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Variable{local: true, name: local},
                    box Node::Variable{local: false, name: global}
                    )) => {
                assert_eq!(&local, "local");
                assert_eq!(&global, "global");
            }
            _ => panic!(),
        }
    }
}
