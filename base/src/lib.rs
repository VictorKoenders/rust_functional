#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod module;

pub use module::{Builder, Config, Instruction, InstructionParameter};