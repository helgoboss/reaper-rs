macro_rules! assert {
    ($condition:expr) => {{
        let result = if ($condition) {
            Ok(())
        } else {
            Err(stringify!($condition))
        };
        result?
    }};
}

macro_rules! assert_eq {
    ($actual:expr, $expected:expr) => {
        match (&$actual, &$expected) {
            (actual_val, expected_val) => {
                let result = if *actual_val == *expected_val {
                    Ok(())
                } else {
                    let actual_expr = stringify!($actual);
                    let expected_expr = stringify!($expected);
                    Err(format!(
                        "\
Expression `{actual_expr}`

```rust
{actual_val:#?}
```

was expected to be equal to expression `{expected_expr}`

```rust
{expected_val:#?}
```",
                        actual_expr = actual_expr,
                        expected_expr = expected_expr,
                        expected_val = &*expected_val,
                        actual_val = &*actual_val
                    ))
                };
                result?
            }
        }
    };
}

macro_rules! assert_ne {
    ($actual:expr, $expected:expr) => {
        match (&$actual, &$expected) {
            (actual_val, expected_val) => {
                let result = if *actual_val != *expected_val {
                    Ok(())
                } else {
                    let actual_expr = stringify!($actual);
                    let expected_expr = stringify!($expected);
                    Err(format!(
                        "\
Expression `{actual_expr}`

```rust
{actual_val:#?}
```

was expected to not be equal to expression `{expected_expr}`

```rust
{expected_val:#?}
```",
                        actual_expr = actual_expr,
                        expected_expr = expected_expr,
                        expected_val = &*expected_val,
                        actual_val = &*actual_val
                    ))
                };
                result?
            }
        }
    };
}
