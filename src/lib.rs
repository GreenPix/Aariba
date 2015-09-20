#![feature(plugin)]
#![plugin(peg_syntax_ext)]

//! A rust library to parse and evaluate arithmetic expressions

#![cfg_attr(test,feature(box_patterns))]

#[macro_use] extern crate log;
extern crate rand;

pub mod expressions;
pub mod parser;
pub mod rules;
