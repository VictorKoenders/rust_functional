extern crate proc_macro2;
extern crate rust_functional;
extern crate syn;

use rust_functional::{Config, Input, Method, NumericConstraint, Output, ParameterType};
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let mut contents = Vec::new();
    let mut file = File::open("modules/adder/src/lib.rs").unwrap();
    file.read_to_end(&mut contents).unwrap();
    let input: syn::File = syn::parse_file(std::str::from_utf8(&contents).unwrap()).unwrap();

    let mut module = Config {
        url: "".into(),
        name: "adder".to_string(),
        description: get_docs(&input.attrs),
        methods: Vec::new(),
    };

    File::create("out.txt")
        .unwrap()
        .write_all(format!("{:#?}", input).as_bytes())
        .unwrap();

    for item in &input.items {
        match item {
            syn::Item::Fn(f) => {
                let mut method = Method {
                    name: format!("{}", f.ident),
                    description: get_docs(&f.attrs),
                    input: Vec::new(),
                    output: Vec::new(),
                };
                if let syn::ReturnType::Type(_, ty) = &f.decl.output {
                    let mut output = Output {
                        name: "result".to_string(),
                        description: "".to_string(),
                        value_type: get_type(ty),
                    };
                    for line in method.description.lines() {
                        if line.starts_with("return:") {
                            output.description = (&line["return:".len()..]).trim().to_string();
                            break;
                        }
                    }
                    method.output.push(output);
                }
                for input in &f.decl.inputs {
                    let mut input = Input {
                        name: get_parameter_name(&input),
                        description: "".to_string(),
                        value_type: get_parameter_type(&input),
                    };
                    let mut line_iter = method.description.lines();
                    while let Some(line) = line_iter.next() {
                        if line.starts_with(&format!("* `{}`:", input.name)) {
                            input.description += line;
                            while let Some(line) = line_iter.next() {
                                if !line.starts_with('*') && !line.trim().is_empty() {
                                    input.description += "\n";
                                    input.description += line;
                                } else {
                                    break;
                                }
                            }
                            break;
                        }
                    }
                    method.input.push(input);
                }
                module.methods.push(method);
            }
            x => {
                panic!("Unexpected item {:?}", x);
            }
        }
    }

    println!("{:#?}", module);
}

fn get_parameter_name(arg: &syn::FnArg) -> String {
    match arg {
        syn::FnArg::Captured(cap) => match &cap.pat {
            syn::Pat::Ident(ident) => format!("{}", ident.ident),
            x => panic!("Unknown capture pattern {:?}", x),
        },
        x => panic!("Unkown parameter type {:?}", x),
    }
}

fn get_parameter_type(arg: &syn::FnArg) -> ParameterType {
    match arg {
        syn::FnArg::Captured(cap) => get_type(&cap.ty),
        x => panic!("Unknown fnarg type: {:?}", x),
    }
}

fn get_type(arg: &syn::Type) -> ParameterType {
    match arg {
        syn::Type::Path(path) => {
            for segment in &path.path.segments {
                let ident = format!("{}", segment.ident);
                if ident == "i32" {
                    return ParameterType::Numeric(NumericConstraint::NoConstraint);
                } else {
                    panic!("Unknown type ident {:?}", ident);
                }
            }
            panic!("{:?}", path);
        }
        syn::Type::Reference(r) => get_type(&*r.elem),
        x => panic!("Unknown type {:?}", x),
    }
}

fn get_docs(attr: &[syn::Attribute]) -> String {
    let mut doc = String::new();
    for attr in attr {
        if is_doc(&attr) {
            for token in attr.tts.clone() {
                if let proc_macro2::TokenTree::Literal(lit) = token {
                    doc += format!("{}", lit)
                        .trim_left_matches("\" ")
                        .trim_left_matches('"')
                        .trim_right_matches('"');
                    doc += "\n";
                }
            }
        }
    }
    doc
}

fn is_doc(attr: &syn::Attribute) -> bool {
    for segment in &attr.path.segments {
        if segment.ident == "doc" {
            return true;
        }
    }
    false
}
