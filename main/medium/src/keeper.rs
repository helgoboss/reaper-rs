use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::Arc;

// Some structs are not self-contained (completely owned). This container takes only
// self-contained structs (T). Those self-contained structs expose the data (R).
#[derive(Debug)]
pub(crate) struct Keeper<T, R> {
    // Maps from a pointer (used as sort of type-safe address/handle) to the struct R that's
    // passed to REAPER when doing e.g. plugin_register(). The owned struct T is boxed in order to
    // obtain a stable memory address offset that survives moving. The address needs to be
    // stable because the data part of it (the part returned by AsRef) will be passed to
    // REAPER.
    map: HashMap<NonNull<R>, Box<T>>,
}

impl<T: AsRef<R>, R> Default for Keeper<T, R> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T: AsRef<R>, R> Keeper<T, R> {
    pub fn keep(&mut self, owned_struct: T) -> NonNull<R> {
        let boxed = Box::new(owned_struct);
        let ref_to_data: &R = (*boxed).as_ref();
        let stable_ptr_to_data: NonNull<R> = ref_to_data.into();
        self.map.insert(stable_ptr_to_data, boxed);
        stable_ptr_to_data
    }

    pub fn release(&mut self, handle: NonNull<R>) -> Option<T> {
        self.map.remove(&handle).map(|boxed| *boxed)
    }
}

#[derive(Debug)]
pub(crate) struct SharedKeeper<T, R> {
    map: HashMap<NonNull<R>, Arc<T>>,
}

impl<T: AsRef<R>, R> Default for SharedKeeper<T, R> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T: AsRef<R>, R> SharedKeeper<T, R> {
    pub fn keep(&mut self, shared: Arc<T>) -> NonNull<R> {
        let ref_to_data: &R = (*shared).as_ref();
        let stable_ptr_to_data: NonNull<R> = ref_to_data.into();
        self.map.insert(stable_ptr_to_data, shared);
        stable_ptr_to_data
    }

    pub fn release(&mut self, handle: NonNull<R>) -> Option<Arc<T>> {
        self.map.remove(&handle)
    }
}
