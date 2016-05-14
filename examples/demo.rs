extern crate aariba;

use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

fn main() {
    let mut args = env::args_os();
    args.next();
    let mut global_variables = HashMap::new();
    for filename in args {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        let evaluator = aariba::parse_rule(&string).unwrap();
        evaluator.evaluate(&mut global_variables).unwrap();
        println!("Evaluation of rules {}\n => {:#?}", string, global_variables);
    }
}
