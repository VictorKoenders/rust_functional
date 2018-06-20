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
    let adder_module = Config::from_path("../rust_functional/modules/adder");
    let actix_helper_module = Config::from_path("modules/actix_web_helper");
    let mut builder = Builder::default();
    builder.add_module(&adder_module);
    builder.add_module(&actix_helper_module);
    builder.add_endpoint({
        let mut endpoint = EndPoint::new("hello_world", "/hello/{name}");
        endpoint.add_base_instruction(BaseInstruction::CallModule {
            config: &actix_helper_module,
            method: "get_property".to_string(),
            out_variable_name: "name".to_string(),
            parameters: vec![
                ("req".to_string(), InstructionParameter::Variable("req".to_string())),
                ("field".to_string(), InstructionParameter::String("name".to_string())),
            ]
        });
        endpoint.add_base_instruction(BaseInstruction::CallModule {
            config: &actix_helper_module,
            method: "format1".to_string(),
            out_variable_name: "result".to_string(),
            parameters: vec![
                ("format".to_string(), InstructionParameter::String("Hello {}".to_string())),
                ("arg0".to_string(), InstructionParameter::Variable("name".to_string())),
            ]
        });
        endpoint.add_instruction(Instruction::Json(InstructionParameter::Variable(
            "result".to_string()
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
