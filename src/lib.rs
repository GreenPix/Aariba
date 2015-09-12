#![feature(plugin)]
#![plugin(peg_syntax_ext)]

//! A rust library to parse and evaluate arithmetic expressions

#[macro_use] extern crate log;

pub mod rules;
pub mod parser;
