use expressions::{ExpressionEvaluator,ExpressionMember,Operator,ValueType};

peg_file! expressions_parser("expressions_rules.peg");

fn combine(left: Node, right_vec: Vec<(Operator,Node)>) -> Node {
    let mut res = left;
    for (op, right) in right_vec {
        match op {
            Operator::Plus => {
                res = Node::Plus((Box::new(res),Box::new(right)));
            }
            Operator::Minus => {
                res = Node::Minus((Box::new(res),Box::new(right)));
            }
            Operator::Multiply => {
                res = Node::Multiply((Box::new(res),Box::new(right)));
            }
            Operator::Divide => {
                res = Node::Divide((Box::new(res),Box::new(right)));
            }
            Operator::Pow => {
                res = Node::Pow((Box::new(res),Box::new(right)));
            }
        }
    }
    res
}

pub type N = Box<Node>;
#[derive(Debug,Clone)]
pub enum Node {
    Plus((N,N)),
    Minus((N,N)),
    Multiply((N,N)),
    Divide((N,N)),
    Pow((N,N)),
    I64(i64),
    F32(f32),
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
                res.push(ExpressionMember::Op(Operator::Plus));
            }
            Node::Minus((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Minus));
            }
            Node::Multiply((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Multiply));
            }
            Node::Divide((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Divide));
            }
            Node::Pow((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Pow));
            }
            Node::I64(num) => {
                res.push(ExpressionMember::Constant(ValueType::I64(num)));
            }
            Node::F32(num) => {
                res.push(ExpressionMember::Constant(ValueType::F32(num)));
            }
            Node::Variable{local,name} => {
                res.push(ExpressionMember::Variable{local:local,name:name});
            }
        }
    }
}

pub fn parse(input: &str) -> Result<ExpressionEvaluator,expressions_parser::ParseError> {
    let tree = try!(expressions_parser::expression(input));
    Ok(convert(tree))
}

#[cfg(test)]
mod tests {
    use super::{Node};
    use super::expressions_parser;
    #[test]
    fn simple_addition() {
        let to_parse = "1 + 2";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Plus((box Node::I64(1), box Node::I64(2))) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn multiple_addition() {
        let to_parse = "1 + 2 + 3";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Plus((box Node::I64(1), box Node::I64(2))),
                    box Node::I64(3)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn substraction_associativity() {
        let to_parse = "1 - 2 + 3";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Minus((box Node::I64(1), box Node::I64(2))),
                    box Node::I64(3)
                    )) => {}
            _ => panic!(),
        }
        let to_parse = "1 + 2 - 3";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Minus((
                    box Node::Plus((box Node::I64(1), box Node::I64(2))),
                    box Node::I64(3)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn priority() {
        let to_parse = "1 + 2 * 3";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::I64(1),
                    box Node::Multiply((box Node::I64(2), box Node::I64(3)))
                    )) => {}
            _ => panic!(),
        }
        let to_parse = "1 * 2 + 3";
        let res = expressions_parser::expression(to_parse).unwrap();
        match res {
            Node::Plus((
                    box Node::Multiply((box Node::I64(1), box Node::I64(2))),
                    box Node::I64(3)
                    )) => {}
            _ => panic!(),
        }
    }
    #[test]
    fn local_global_variables() {
        let to_parse = "local + $global";
        let res = expressions_parser::expression(to_parse).unwrap();
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
