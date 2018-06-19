#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate rust_functional;
extern crate serde_json;

use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::process::Command;

use rust_functional::{Builder, Config, Instruction, InstructionParameter};

fn main() {
    let _ = remove_dir_all("output");
    let config = Config::from_path("modules/adder");

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

    builder.add_instruction(Instruction::Exit(InstructionParameter::Variable(
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
    let mut dir = std::env::current_dir().unwrap();
    dir.push("modules");
    dir.push("adder");
    let dir = dir.to_str().unwrap().replace("\\", "/");
    assert_eq!(
        format!(r#"[package]
name = "test"
version = "0.1.0"
authors = [""]

[dependencies]
adder = {{ path = "{}" }}
"#, dir),
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
