use super::ParameterType;

#[derive(Debug, Deserialize)]
pub struct Input {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub value_type: ParameterType,
}
