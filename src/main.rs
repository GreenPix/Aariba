extern crate expression_evaluator;

use std::collections::HashMap;

#[cfg(not(test))]
fn main() {
    let to_parse = "1 - 2 + 6 / 2 ^ 3";

    let res = expression_evaluator::parser::parse_expression(to_parse).unwrap();
    println!("Parsed into {:?}", res);
    let global = HashMap::new();
    let local = HashMap::new();
    println!("{} = {:?}", to_parse, res.evaluate(&global,&local).unwrap());
}
