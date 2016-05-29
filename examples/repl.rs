extern crate aariba;

use std::io::{self,BufRead};
use std::collections::HashMap;
use std::fmt::Write;

fn main() {
    let stdin = io::stdin();
    let mut accumulated_rules = String::new();
    for line in stdin.lock().lines().filter_map(|l| l.ok()) {
        match line.trim() {
            "clear;" => accumulated_rules.clear(),
            _ => {
                let mut new_rules = String::new();
                write!(new_rules, "{}{}\n", accumulated_rules, line).unwrap();
                println!("Evaluating the following rules:\n{}", new_rules);
                let res = aariba::parse_rule(&new_rules);
                match res {
                    Ok(evaluator) => {
                        let mut global_variables = HashMap::new();
                        match evaluator.evaluate(&mut global_variables) {
                            Ok(()) => {
                                println!("Global variables: {:#?}", global_variables);
                                accumulated_rules = new_rules;
                            }
                            Err(e) => {
                                println!("Evaluation error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Parsing Error: {}", e);
                    }
                }
            }
        }
    }
}
