use regex::Regex;

#[derive(Debug, Clone, Serialize)]
pub enum StringConstraint {
    NoConstraint,
    Regex(
        #[serde(serialize_with = "serde_regex")]
        Regex
    ),
    StringList(Vec<String>),
}

fn serde_regex<S>(regex: &Regex, s: S) -> Result<S::Ok, S::Error>
    where S: ::serde::Serializer {
        s.serialize_str(&format!("{}", regex))
}