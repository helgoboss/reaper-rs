use std::cell::{Cell, RefCell, Ref};
use std::rc::Rc;

pub struct InvocationMock<O: Clone> {
    count: Cell<u32>,
    last_arg: RefCell<Option<O>>,
}

impl<O: Clone> InvocationMock<O> {
    pub fn invoke(&self, arg: O) {
        self.count.replace(self.count.get() + 1);
        self.last_arg.replace(Some(arg));
    }

    pub fn invocation_count(&self) -> u32 {
        self.count.get()
    }

    pub fn last_arg(&self) -> O {
        self.last_arg.borrow().clone().expect("There were no invocations")
    }
}

pub fn observe_invocations<O: Clone, R>(op: impl FnOnce(Rc<InvocationMock<O>>) -> R) -> (Rc<InvocationMock<O>>, R) {
    let mock = InvocationMock {
        count: Cell::new(0),
        last_arg: RefCell::new(None),
    };
    let shareable_mock = Rc::new(mock);
    let mirrored_mock = shareable_mock.clone();
    (mirrored_mock, op(shareable_mock))
}
