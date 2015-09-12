use expressions::{ExpressionEvaluator,ExpressionMember,Operator,ValueType};

peg_file! parser("parser_rules.peg");

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

/*
impl Prio2 {
    fn convert(self, res: &mut Vec<ExpressionMember>) {
        match self {
            Prio2::Multiply((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Multiply));
            }
            Prio2::Divide((l,r)) => {
                l.convert(res);
                r.convert(res);
                res.push(ExpressionMember::Op(Operator::Divide));
            }
            Prio2::None(p) => p.convert(res),
        }
    }
}

impl Prio3 {
    fn convert(self, res: &mut Vec<ExpressionMember>) {
        match self {
            Prio3::Pow((l,r)) => {
                l.convert(res);
                r.convert(res);
                unimplemented!();
                //res.push(ExpressionMember::Op(Operator::Plus));
            }
            Prio3::None(p) => p.convert(res),
        }
    }
}

impl Prio4 {
    fn convert(self, res: &mut Vec<ExpressionMember>) {
        match self {
            Prio4::I64(num) => {
                res.push(ExpressionMember::Constant(ValueType::I64(num)));
            }
            Prio4::F32(num) => {
                res.push(ExpressionMember::Constant(ValueType::F32(num)));
            }
            Prio4::Variable(name) => {
                res.push(ExpressionMember::Variable(name));
            }
            Prio4::Prio1(p) => p.convert(res),
        }
    }
}
*/
pub fn parse(input: &str) -> Result<ExpressionEvaluator,parser::ParseError> {
    let tree = try!(parser::expression(input));
    Ok(convert(tree))
}

#[cfg(test)]
mod tests {
    use super::{Prio1,Prio2,Prio3,Prio4};
    use super::parser;
    #[test]
    fn simple_addition() {
        let to_parse = "1 + 2";
        let res = parser::expression(to_parse).unwrap();
        match res {
            Prio1::Plus((Prio2::None(Prio3::None(Prio4::I64(1))),
                         Prio2::None(Prio3::None(Prio4::I64(2))))) => {}
            _ => panic!(),
        }
    }
}
