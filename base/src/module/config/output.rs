use super::ParameterType;

#[derive(Debug, Deserialize, Serialize)]
pub struct Output {
    pub name: String,
    pub description: String,
    #[serde(default, rename = "type")]
    pub value_type: ParameterType,
}
