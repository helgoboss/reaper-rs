pub fn is_normalized_value(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}
