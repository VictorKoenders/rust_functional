use instruction::Instruction;
use rust_functional::{Config, Instruction as BaseInstruction};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Builder {
    modules: Vec<Rc<Config>>,
    endpoints: Vec<EndPoint>,
}

impl Builder {
    pub fn add_module(&mut self, module: Rc<Config>) {
        if self
            .modules
            .iter()
            .any(|m| Rc::ptr_eq(m, &module))
        {
            return;
        }
        self.modules.push(module);
    }

    pub fn add_endpoint(&mut self, endpoint: EndPoint) {
        self.endpoints.push(endpoint);
    }

    pub fn build(self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        result.insert("Cargo.toml".to_string(), {
            let mut str = r#"[package]
name = "test"
version = "0.1.0"
authors = [""]

[dependencies]
actix-web = "*"
"#.to_string();
            for module in &self.modules {
                str += &format!(
                    "{} = {{ path = \"{}\" }}\n",
                    module.name,
                    module.url.to_str().unwrap().replace("\\", "/")
                );
            }
            str += r#"
[replace]
"cookie:0.10.1" = { path = "../libs/cookie-rs" }
"#;
            str
        });

        result.insert("src/main.rs".to_string(), {
            let mut str = "extern crate actix_web;\n".to_string();
            for module in &self.modules {
                str += &format!("extern crate {};\n", module.name);
            }
            str += r#"
fn main() {
    actix_web::server::new(||
        actix_web::App::new()
"#;
            for endpoint in &self.endpoints {
                str += &endpoint.register_with_app();
            }
            str += r#"
    )
    .bind("127.0.0.1:8080").unwrap()
    .run();
}
"#;

            for endpoint in &self.endpoints {
                str += &endpoint.create_function();
            }

            str
        });

        result
    }
}

#[derive(Debug, Default)]
pub struct EndPoint {
    pub name: String,
    pub url: String,
    pub instructions: Vec<Instruction>,
}

impl EndPoint {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> EndPoint {
        EndPoint {
            name: name.into(),
            url: url.into(),
            instructions: Vec::new(),
        }
    }

    pub fn add_base_instruction(&mut self, instruction: BaseInstruction) {
        self.instructions
            .push(Instruction::BaseInstruction(instruction));
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn register_with_app(&self) -> String {
        format!(
            "            .route({:?}, actix_web::http::Method::GET, {})",
            self.url, self.name
        )
    }

    fn create_function(&self) -> String {
        let mut result = String::new();
        result += &format!(
            "\nfn {}(req: actix_web::HttpRequest) -> impl actix_web::Responder {{\n",
            self.name
        );
        for instruction in &self.instructions {
            result += &instruction.build();
        }
        result += "}\n";
        result
    }
}
