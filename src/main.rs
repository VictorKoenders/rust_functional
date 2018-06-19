#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate regex;

mod module;

use std::fs::File;

fn main() {
    let mut file = File::open("module.json").unwrap();
    let config: module::config::Config = serde_json::from_reader(&mut file).unwrap();
    println!("{:#?}", config);
}
