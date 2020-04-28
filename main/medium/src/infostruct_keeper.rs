use std::collections::HashMap;
use std::ptr::NonNull;

// Many infostructs are not self-contained (completely owned). This container takes only
// self-contained things (T). Those self-contained things expose the infostructs (R).
pub struct InfostructKeeper<T: AsRef<R>, R> {
    // Maps from a pointer (used as sort of type-safe address/handle) to the struct R that's
    // passed to REAPER when doing plugin_register(). The owned struct T is boxed in order to
    // obtain a stable memory address offset that survives moving. The address needs to be
    // stable because the infostruct part of it (the part returned by AsRef) will be passed to
    // REAPER.
    map: HashMap<NonNull<R>, Box<T>>,
}

impl<T: AsRef<R>, R> Default for InfostructKeeper<T, R> {
    fn default() -> Self {
        InfostructKeeper {
            map: Default::default(),
        }
    }
}

impl<T: AsRef<R>, R> InfostructKeeper<T, R> {
    pub fn keep(&mut self, owned_struct: T) -> NonNull<R> {
        let boxed = Box::new(owned_struct);
        let ref_to_infostruct: &R = (*boxed).as_ref();
        let stable_ptr_to_infostruct: NonNull<R> = ref_to_infostruct.into();
        self.map.insert(stable_ptr_to_infostruct, boxed);
        stable_ptr_to_infostruct
    }

    pub fn release(&mut self, handle: NonNull<R>) -> Option<T> {
        self.map.remove(&handle).map(|boxed| *boxed)
    }

    pub fn release_all(&mut self) -> Vec<NonNull<R>> {
        self.map.drain().map(|(handle, _)| handle).collect()
    }
}
