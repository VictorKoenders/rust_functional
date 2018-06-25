extern crate rust_functional;

mod builder;
mod instruction;

use builder::{Builder, EndPoint};
use instruction::Instruction;
use rust_functional::{Config, Instruction as BaseInstruction, InstructionParameter};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::process::Command;

fn main() {
    let postgres = Config::from_path("modules/postgres");
    let mut builder = Builder::default();
    builder.add_module(&postgres);
    builder.add_endpoint({
        let mut endpoint = EndPoint::new("user_list", "/api/users/list");
        endpoint.add_base_instruction(BaseInstruction::CallModule {
            config: &postgres,
            method: "get_connection".to_string(),
            out_variable_name: "connection".to_string(),
            parameters: vec![],
        });
        endpoint.add_base_instruction(BaseInstruction::CallModule {
            config: &postgres,
            method: "execute_query".to_string(),
            out_variable_name: "result".to_string(),
            parameters: vec![
                (
                    "connection".to_string(),
                    InstructionParameter::Variable("connection".to_string()),
                ),
                (
                    "query".to_string(),
                    InstructionParameter::String("SELECT * FROM users".to_string()),
                ),
            ],
        });
        endpoint.add_instruction(Instruction::Json(InstructionParameter::Variable(
            "result".to_string(),
        )));
        endpoint
    });

    let files = builder.build();

    create_dir_all("output/src").unwrap();
    for (name, content) in files {
        File::create(format!("output/{}", name))
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    }

    let mut dir = ::std::env::current_dir().unwrap();
    dir.push("output");

    let output = Command::new("cargo")
        .arg("run")
        .current_dir(dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    println!("{:#?}", output);
}
