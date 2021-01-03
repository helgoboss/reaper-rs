pub trait OptionExt {
    type T;

    // TODO-low Replace with https://github.com/rust-lang/rust/issues/62358 as soon as stable.
    fn contains(&self, x: &Self::T) -> bool
    where
        Self::T: PartialEq;
}

impl<T> OptionExt for Option<T> {
    type T = T;

    fn contains(&self, x: &Self::T) -> bool
    where
        Self::T: PartialEq,
    {
        match self {
            Some(y) => y == x,
            None => false,
        }
    }
}
