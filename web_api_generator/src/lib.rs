extern crate rust_functional;

mod builder;
mod instruction;

pub use builder::{Builder, EndPoint};
pub use instruction::Instruction;
pub use rust_functional::{Config, Instruction as BaseInstruction, InstructionParameter};
