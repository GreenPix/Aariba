//! A rust library to parse and evaluate arithmetic expressions

#![cfg_attr(test,feature(box_patterns))]

#[macro_use] extern crate log;
extern crate rand;

pub mod expressions;
mod parser;
pub mod rules;
pub mod conditions;

pub use self::parser::parse_rule;
