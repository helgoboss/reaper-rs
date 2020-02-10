macro_rules! check {
    ($condition:expr) => {
        {
            let result = if ($condition) {
                Ok(())
            } else {
                Err(stringify!($condition))
            };
            result?
        }
    }
}

macro_rules! check_eq {
    ($actual:expr, $expected:expr) => {
        {
            let actual = $actual;
            let expected = $expected;
            let result = if (actual == expected) {
                Ok(())
            } else {
                let actual_expr = stringify!($actual);
                let expected_expr = stringify!($expected);
                Err(format!("[{}] was expected to be [{}] but is {:?}", actual_expr, expected_expr, actual))
            };
            result?
        }
    }
}

macro_rules! check_ne {
    ($actual:expr, $unexpected:expr) => {
        {
            let actual = $actual;
            let unexpected = $unexpected;
            let result = if (actual == unexpected) {
                let actual_expr = stringify!($actual);
                let unexpected_expr = stringify!($unexpected);
                Err(format!("[{}] was expected to not be [{}] but it is ({:?})", actual_expr, unexpected_expr, actual))
            } else {
                Ok(())
            };
            result?
        }
    }
}