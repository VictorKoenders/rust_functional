use super::config::Config;
use std::collections::HashMap;

#[derive(Default)]
pub struct Builder<'a> {
    modules: Vec<&'a Config>,
    instructions: Vec<Instruction<'a>>,
}

impl<'a> Builder<'a> {
    pub fn add_module(&mut self, module: &'a Config) {
        if self
            .modules
            .iter()
            .any(|m| ::std::ptr::eq(*m as *const Config, module as *const Config))
        {
            return;
        }
        self.modules.push(module);
    }

    pub fn add_instruction(&mut self, instruction: Instruction<'a>) {
        self.instructions.push(instruction);
    }

    pub fn build(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("src/main.rs".to_string(), {
            let mut result = String::new();
            for module in &self.modules {
                result += "extern crate ";
                result += &module.name;
                result += ";\n";
            }

            result += "\n";
            result += "fn main() {\n";
            for instruction in &self.instructions {
                result += &instruction.build();
            }
            result += "}";
            result
        });
        map.insert("Cargo.toml".to_string(), {
            let mut result = r#"[package]
name = "test"
version = "0.1.0"
authors = [""]

[dependencies]
"#.to_string();
            for module in &self.modules {
                result += &format!(
                    "{} = {{ path = \"{}\" }}\n",
                    module.name,
                    module.url.to_str().unwrap().replace("\\", "/")
                );
            }
            result
        });
        map
    }
}

#[derive(Debug)]
pub enum Instruction<'a> {
    CallModule {
        config: &'a Config,
        method: String,
        parameters: Vec<(String, InstructionParameter)>,
        out_variable_name: String,
    },
    Return(InstructionParameter),
    Exit(InstructionParameter),
}

impl<'a> Instruction<'a> {
    pub fn build(&self) -> String {
        match self {
            Instruction::CallModule {
                config,
                method,
                parameters,
                out_variable_name,
            } => {
                let method = config
                    .methods
                    .iter()
                    .find(|m| &m.name == method)
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not find method {:?}, available: {:?}",
                            method,
                            config.methods.iter().map(|m| &m.name).collect::<Vec<_>>()
                        )
                    });
                let mut args = Vec::with_capacity(method.input.len());
                for arg in &method.input {
                    let value = parameters
                        .iter()
                        .find(|p| p.0 == arg.name)
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find parameter {:?}, provided: {:?}",
                                arg.name,
                                parameters.iter().map(|p| &p.0).collect::<Vec<_>>()
                            )
                        });
                    args.push(format!("&{}", value.1.to_string()));
                }
                format!(
                    "    let {} = {}::{}({});\n",
                    out_variable_name.to_string(),
                    config.name,
                    method.name,
                    args.join(", ")
                )
            }
            Instruction::Exit(param) => format!("    std::process::exit({});\n", param.to_string()),
            Instruction::Return(param) => format!("    {}\n", param.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum InstructionParameter {
    Variable(String),
    String(String),
    Number(i32),
    Float(f32),
}

impl InstructionParameter {
    pub fn to_string(&self) -> String {
        match self {
            InstructionParameter::Variable(name) => name.clone(),
            InstructionParameter::String(value) => format!("{:?}", value),
            InstructionParameter::Number(value) => value.to_string(),
            InstructionParameter::Float(value) => value.to_string(),
        }
    }
}
