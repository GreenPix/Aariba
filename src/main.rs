extern crate math_rules;

use std::collections::HashMap;

fn main() {
    let to_parse = "1 - 2 + 6 / 2 ^ 3";

    let res = math_rules::parser::parse(to_parse).unwrap();
    let context = HashMap::new();
    println!("Parsed into {:?}", res);
    println!("{} = {:?}", to_parse, res.evaluate(&context).unwrap());
}
