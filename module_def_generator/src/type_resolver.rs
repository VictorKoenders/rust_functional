use rust_functional::{NumericConstraint, ParameterType, StringConstraint};
use std::collections::HashMap;
use syn;

#[derive(Debug, Clone)]
pub struct TypeResolver {
    modules: Vec<Module>,
    types: HashMap<String, ParameterType>,
}

impl Default for TypeResolver {
    fn default() -> Self {
        TypeResolver {
            modules: Vec::new(),
            types: {
                let mut map = HashMap::new();
                map.insert(
                    "::i32".to_string(),
                    ParameterType::Numeric(NumericConstraint::NoConstraint),
                );
                map.insert(
                    "::String".to_string(),
                    ParameterType::String(StringConstraint::NoConstraint),
                );
                map.insert(
                    "::str".to_string(),
                    ParameterType::String(StringConstraint::NoConstraint),
                );
                map
            },
        }
    }
}

impl TypeResolver {
    pub fn register_module(&mut self, name: String, alias: Option<String>) {
        self.modules.push(Module { name, alias });
    }

    pub fn get_type(&self, ty: &syn::Type) -> ParameterType {
        match ty {
            syn::Type::Path(path) => {
                let ident = path
                    .path
                    .segments
                    .iter()
                    .map(|s| format!("::{}", s.ident.to_string()))
                    .collect::<String>();
                if let Some(param) = self.types.get(&ident) {
                    return param.clone();
                }
                println!("Unknown type ident {:?}, assuming object", ident);
                ParameterType::Object(ident)
            }
            syn::Type::Reference(r) => self.get_type(&*r.elem),
            syn::Type::ImplTrait(i) => {
                let mut bounds = Vec::with_capacity(i.bounds.len());
                for bound in &i.bounds {
                    if let syn::TypeParamBound::Trait(t) = bound {
                        let ident = t
                            .path
                            .segments
                            .iter()
                            .map(|s| format!("::{}", s.ident.to_string()))
                            .collect::<String>();
                        bounds.push(ident);
                    }
                }
                ParameterType::Trait(bounds)
            }
            x => panic!("Unknown type {:?}", x),
        }
    }

    pub fn get_fn_arg_type(&self, arg: &syn::FnArg) -> ParameterType {
        match arg {
            syn::FnArg::Captured(cap) => self.get_type(&cap.ty),
            x => panic!("Unknown fnarg type: {:?}", x),
        }
    }
}

#[derive(Debug, Clone)]
struct Module {
    name: String,
    alias: Option<String>,
}
