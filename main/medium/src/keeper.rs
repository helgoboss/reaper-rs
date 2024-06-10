use crate::Handle;
use std::collections::HashMap;
use std::sync::Arc;

/// A very simple collection that keeps completely owned values in memory, identified by an auto-generated ID (handle)
/// that can later be used to remove the value from the collection.
#[derive(Debug)]
pub(crate) struct SimpleKeeper<T> {
    items: HashMap<usize, Box<T>>,
    next_id: usize,
}

impl<T> Default for SimpleKeeper<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            next_id: 0,
        }
    }
}

impl<T> SimpleKeeper<T> {
    pub fn keep(&mut self, item: T) -> (usize, &T) {
        let id = self.next_id;
        self.items.insert(id, Box::new(item));
        self.next_id += 1;
        let added_item = self.get(id).unwrap();
        (id, added_item)
    }

    pub fn get(&self, handle: usize) -> Option<&T> {
        let boxed = self.items.get(&handle)?;
        Some(boxed.as_ref())
    }

    pub fn release(&mut self, handle: usize) -> Option<T> {
        let boxed = self.items.remove(&handle)?;
        Some(*boxed)
    }
}

/// Like [`SimpleKeeper`] but the handle is a pointer to a REAPER-specific struct, not just an arbitrary integer.
///
/// Some structs are not self-contained (completely owned). This container takes only
/// self-contained structs (T). Those self-contained structs expose the data (R).
#[derive(Debug)]
pub(crate) struct Keeper<T, R> {
    // Maps from a pointer (used as sort of type-safe address/handle) to the struct R that's
    // passed to REAPER when doing e.g. plugin_register(). The owned struct T is boxed in order to
    // obtain a stable memory address offset that survives moving. The address needs to be
    // stable because the data part of it (the part returned by AsRef) will be passed to
    // REAPER.
    map: HashMap<Handle<R>, Box<T>>,
}

impl<T: AsRef<R>, R> Default for Keeper<T, R> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T: AsRef<R>, R> Keeper<T, R> {
    pub fn keep(&mut self, owned_struct: T) -> Handle<R> {
        let boxed = Box::new(owned_struct);
        let ref_to_data: &R = (*boxed).as_ref();
        let stable_ptr_to_data: Handle<R> = Handle::new(ref_to_data.into());
        self.map.insert(stable_ptr_to_data, boxed);
        stable_ptr_to_data
    }

    pub fn release(&mut self, handle: Handle<R>) -> Option<T> {
        self.map.remove(&handle).map(|boxed| *boxed)
    }
}

/// Like [`Keeper`] but allows for shared ownership between REAPER and the plug-in.
#[derive(Debug)]
pub(crate) struct SharedKeeper<T, R> {
    map: HashMap<Handle<R>, Arc<T>>,
}

impl<T: AsRef<R>, R> Default for SharedKeeper<T, R> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T: AsRef<R>, R> SharedKeeper<T, R> {
    pub fn keep(&mut self, shared: Arc<T>) -> Handle<R> {
        let ref_to_data: &R = (*shared).as_ref();
        let stable_ptr_to_data = Handle::new(ref_to_data.into());
        self.map.insert(stable_ptr_to_data, shared);
        stable_ptr_to_data
    }

    pub fn release(&mut self, handle: Handle<R>) -> Option<Arc<T>> {
        self.map.remove(&handle)
    }
}
