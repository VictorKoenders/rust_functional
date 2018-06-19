mod constraints;
mod input;
mod output;
mod parameter_type;

pub use self::constraints::{NumericConstraint, StringConstraint};
pub use self::input::Input;
pub use self::output::Output;
pub use self::parameter_type::ParameterType;
use serde::de::SeqAccess;

use serde::{Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};
use serde::de::value::MapAccessDeserializer;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
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
