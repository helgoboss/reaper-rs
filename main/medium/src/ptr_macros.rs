macro_rules! define_ptr_wrapper {
    ($name: ident, $ptr_type: path) => {
        // The contained pointer is non-null.
        // The advantage over using NonNull<T> is that we can offer medium-level methods on the
        // pointers.
        //
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name(*mut $ptr_type);

        impl $name {
            pub(super) fn required(ptr: *mut $ptr_type) -> Result<$name, ()> {
                if ptr.is_null() {
                    Err(())
                } else {
                    Ok($name(ptr))
                }
            }

            pub(super) fn required_panic(ptr: *mut $ptr_type) -> $name {
                if ptr.is_null() {
                    panic!("MediaTrack unexpectedly null");
                }
                $name(ptr)
            }

            pub(super) fn optional(ptr: *mut $ptr_type) -> Option<$name> {
                if ptr.is_null() {
                    None
                } else {
                    Some($name(ptr))
                }
            }
        }

        // This is for easy extraction of the raw pointer. First and foremost for the medium-level
        // API implementation code (because it needs to call the low-level API). But also for
        // consumers who need to resort to the low-level API. However, once one starts using the
        // low-level API and gets a pointer from it, they can't use that pointer in safe
        // medium-level methods. That's by design.
        impl From<$name> for *mut $ptr_type {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        // This is for easy extraction of the raw pointer as c_void.
        impl From<$name> for *mut std::os::raw::c_void {
            fn from(v: $name) -> Self {
                v.0 as *mut std::os::raw::c_void
            }
        }
    };
}
