#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod module;

use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::process::Command;

use module::{Builder, Config, Instruction, InstructionParameter};

fn main() {
    let _ = remove_dir_all("output");
    let mut file = File::open("modules/adder/module.json").unwrap();
    let config: Config = serde_json::from_reader(&mut file).unwrap();

    let mut builder = Builder::default();
    builder.add_module(&config);
    builder.add_instruction(Instruction::CallModule {
        config: &config,
        method: "add".to_string(),
        parameters: vec![
            ("a".to_string(), InstructionParameter::Number(5)),
            ("b".to_string(), InstructionParameter::Number(10)),
        ],
        out_variable_name: "out".to_string(),
    });

    builder.add_instruction(Instruction::Return(InstructionParameter::Variable(
        "out".to_string(),
    )));

    let files = builder.build();

    assert_eq!(
        r#"extern crate adder;

fn main() {
    let out = adder::add(5, 10);
    std::process::exit(out);
}"#,
        files["src/main.rs"]
    );
    assert_eq!(
        r#"[package]
name = "test"
version = "0.1.0"
authors = [""]

[dependencies]
adder = { path = "../modules/adder" }
"#,
        files["Cargo.toml"]
    );

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
        .output()
        .unwrap();

    assert_eq!(Some(15), output.status.code());
}
