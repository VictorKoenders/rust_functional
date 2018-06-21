use regex::Regex;

#[derive(Debug, Clone)]
pub enum StringConstraint {
    NoConstraint,
    Regex(Regex),
    StringList(Vec<String>),
}
