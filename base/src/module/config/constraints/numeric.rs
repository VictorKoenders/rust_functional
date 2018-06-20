
#[derive(Debug)]
pub enum NumericConstraint {
    NoConstraint,
    IntegerRange { from: i32, to: i32 },
    DecimalRange { from: f32, to: f32 },
    IntegerList(Vec<i32>),
}
