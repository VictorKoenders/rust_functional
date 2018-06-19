use super::{NumericConstraint, StringConstraint};
use serde::{de::MapAccess, de::Visitor, Deserialize, Deserializer};
use std::fmt;

#[derive(Debug)]
pub enum ParameterType {
    Unknown,
    Numeric(NumericConstraint),
    String(StringConstraint),
}

impl<'de> Deserialize<'de> for ParameterType {
    fn deserialize<D>(deserializer: D) -> Result<ParameterType, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ParameterTypeVisitor)
    }
}

struct ParameterTypeVisitor;

impl<'de> Visitor<'de> for ParameterTypeVisitor {
    type Value = ParameterType;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Expected one of types: numeric, string")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut parameter_type = ParameterType::Unknown;
        while let Some(key) = map.next_key::<String>()? {
            if key == "type" {
                match map.next_value::<String>()?.to_lowercase().as_str() {
                    "number" | "numeric" => {
                        if let ParameterType::Unknown = parameter_type {
                            parameter_type = ParameterType::Numeric(NumericConstraint::NoConstraint);
                        } else if let ParameterType::Numeric(_) = parameter_type {
                            // do nothing
                        } else {
                            panic!("Could not set parameter type to numeric, conflicting information");
                        }
                    }
                    "string" | "text" => {
                        if let ParameterType::Unknown = parameter_type {
                            parameter_type = ParameterType::String(StringConstraint::NoConstraint);
                        } else if let ParameterType::String(_) = parameter_type {
                            // do nothing
                        } else {
                            panic!("Could not set parameter type to string, conflicting information");
                        }
                    }
                    x => {
                        println!("Unexpected value {:?}", x);
                        panic!();
                    }
                }
            } else if key == "between" {
                let value: Vec<i32> = map.next_value()?;
                let value = NumericConstraint::IntegerRange { from: value[0], to: value[1] };
                if let ParameterType::Numeric(ref mut constraint) = parameter_type {
                    *constraint = value;
                } else if let ParameterType::Unknown = parameter_type {
                    parameter_type = ParameterType::Numeric(value);
                } else {
                    panic!("Could not set 'between' property on parameter type");
                }
            } else {
                println!("Unexpected key {:?}", key);
                panic!();
            }
        }
        Ok(parameter_type)
    }
}
