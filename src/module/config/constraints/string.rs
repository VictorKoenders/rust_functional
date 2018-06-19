use regex::Regex;

#[derive(Debug)]
pub enum StringConstraint {
    NoConstraint,
    Regex(Regex),
    StringList(Vec<String>),
}
