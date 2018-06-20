use rust_functional::{Instruction as BaseInstruction, InstructionParameter};

#[derive(Debug)]
pub enum Instruction<'a> {
    BaseInstruction(BaseInstruction<'a>),
    Json(InstructionParameter)
}

impl<'a> Instruction<'a> {
    pub fn build(&self) -> String {
        match self {
            Instruction::BaseInstruction(bi) => bi.build(),
            Instruction::Json(param) => format!("    actix_web::Json({})\n", param.to_string())
        }
    }
}
