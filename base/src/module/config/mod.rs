mod constraints;
mod input;
mod output;
mod parameter_type;

pub use self::constraints::{NumericConstraint, StringConstraint};
pub use self::input::Input;
pub use self::output::Output;
pub use self::parameter_type::ParameterType;
use serde::de::SeqAccess;

use serde::de::value::MapAccessDeserializer;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip)]
    pub url: PathBuf,

    pub name: String,
    pub description: String,

    pub methods: Vec<Method>,
}

impl Config {
    pub fn from_path(p: &str) -> Config {
        let mut path = ::std::env::current_dir().unwrap();
        path.push(p);
        let mut json_file = path.clone();
        json_file.push("module.json");
        let mut file = File::open(&json_file).unwrap();
        let mut config: Config = ::serde_json::from_reader(&mut file).unwrap();
        config.url = path;
        config
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Method {
    pub name: String,
    pub description: String,

    #[serde(deserialize_with = "array_or_single")]
    pub input: Vec<Input>,

    #[serde(deserialize_with = "array_or_single")]
    pub output: Vec<Output>,
}

fn array_or_single<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct ArrayOrSingle<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for ArrayOrSingle<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sequence or map")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = if let Some(size) = seq.size_hint() {
                Vec::with_capacity(size)
            } else {
                Vec::new()
            };
            while let Some(item) = seq.next_element()? {
                result.push(item);
            }
            Ok(result)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut result = Vec::with_capacity(1);
            result.push(T::deserialize(MapAccessDeserializer::new(map))?);
            Ok(result)
        }
    }

    deserializer.deserialize_any(ArrayOrSingle(PhantomData))
}
