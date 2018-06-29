use rust_functional::{Instruction as BaseInstruction, InstructionParameter};

#[derive(Debug)]
pub enum Instruction {
    BaseInstruction(BaseInstruction),
    Json(InstructionParameter),
}

impl Instruction {
    pub fn build(&self) -> String {
        match self {
            Instruction::BaseInstruction(bi) => bi.build(),
            Instruction::Json(param) => {
                format!("    actix_web::Json({})\n", param.to_string(false))
            }
        }
    }
}
