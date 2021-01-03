use rxrust::prelude::*;
use std::cell::RefCell;

pub trait ReactiveEvent<T> = Observable<Item = T> + LocalObservable<'static, Err = ()> + 'static;

// This is a RefCell. So calling next() while another next() is still running will panic.
// I guess it's good that way because this is very generic code, panicking or not panicking
// depending on the user's code. And getting a panic is good for becoming aware of the problem
// instead of running into undefined behavior. The developer can always choose to defer to
// the next `ControlSurface::run()` invocation (execute things in next main loop cycle).
//
// Mutex is not necessary because control surface methods are called from main thread only.
pub(crate) type EventStreamSubject<T> = RefCell<LocalSubject<'static, T, ()>>;
