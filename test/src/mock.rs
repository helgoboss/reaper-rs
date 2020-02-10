use std::cell::Cell;
use std::rc::Rc;

pub struct InvocationMock<O: Copy> {
    count: Cell<u32>,
    last_arg: Cell<Option<O>>,
}

impl<O: Copy> InvocationMock<O> {
    pub fn invoke(&self, arg: O) {
        self.count.replace(self.count.get() + 1);
        self.last_arg.replace(Some(arg));
    }

    pub fn invocation_count(&self) -> u32 {
        self.count.get()
    }

    pub fn last_arg(&self) -> O {
        self.last_arg.get().expect("There were no invocations")
    }
}

pub fn observe_invocations<O: Copy>(op: impl FnOnce(Rc<InvocationMock<O>>)) -> Rc<InvocationMock<O>> {
    let mock = InvocationMock {
        count: Cell::new(0),
        last_arg: Cell::new(None),
    };
    let shareable_mock = Rc::new(mock);
    let mirrored_mock = shareable_mock.clone();
    op(shareable_mock);
    mirrored_mock
}
