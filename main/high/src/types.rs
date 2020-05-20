use rxrust::prelude::*;

pub trait ReactiveEvent<T> = Observable<Item = T> + LocalObservable<'static, Err = ()> + 'static;
