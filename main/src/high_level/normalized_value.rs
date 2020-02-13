use std::ops::RangeInclusive;

pub fn is_normalized_value(value: f64) -> bool {
    0.0 <= value && value <= 1.0
}