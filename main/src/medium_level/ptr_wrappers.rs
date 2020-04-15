use crate::low_level::raw;

macro_rules! define_ptr_wrapper {
    ($name: ident, $ptr_type: path) => {
        // Only medium-level API can create such objects. This is the most important point why we
        // don't need `unsafe` for medium-level API methods that take such objects instead of
        // pointers. Because only if we don't let anyone create such objects, we can safely assume
        // that this is really a pointer of that type and has not been messed with (by pointer
        // casting - which can be made even with unsafe code!). The contained pointer is non-null.
        //
        // We obtain this object directly from REAPER and we can't
        // give it a sane lifetime annotation. It's "rather" static from the perspective of the
        // plug-in, yet it could come and go anytime, so 'static would be too optimistic. Annotating
        // with a lifetime 'a - correlated to another lifetime - would be impossible because we
        // don't have such another lifetime which can serve as frame of reference. So the best we
        // can do is making a simple pointer wrapper. The advantage over using the raw pointer is
        // that we can use Option instead of null (making signatures much better) and that we can
        // offer methods on the pointers if necessary.
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
        impl From<$name> for *mut std::ffi::c_void {
            fn from(v: $name) -> Self {
                v.0 as *mut std::ffi::c_void
            }
        }
    };
}

// One of the responsibilities of the medium-level API is to use identifiers which follow the Rust
// conventions. It just happens that some of the C++ classes already conform to Rust conventions,
// so we won't rename them.
define_ptr_wrapper!(MediaTrack, raw::MediaTrack);
define_ptr_wrapper!(ReaProject, raw::ReaProject);
define_ptr_wrapper!(TrackEnvelope, raw::TrackEnvelope);
// Even we create IReaperControlSurface instances ourselves (not REAPER), we don't do it on
// Rust side but on C++ side. So a pointer wrapper is the right way to go here as well. We also
// remove the I from the name because it's not following Rust conventions.
define_ptr_wrapper!(ReaperControlSurface, raw::IReaperControlSurface);
// This is unlike MediaTrack and Co. in that it points to a struct which is *not* opaque. Still, we
// need it as pointer and it has the same lifetime characteristics. The difference is that we add
// type-safe methods to it to lift the possibilities in the struct to medium-level API style. This
// is similar to our midi_Input wrapper in low-level REAPER (just that it doesn't lift the API to
// medium-level API style but restores low-level functionality).
define_ptr_wrapper!(KbdSectionInfo, raw::KbdSectionInfo);

impl KbdSectionInfo {
    // TODO-high Should we make this unsafe? I think this is no different than with other functions
    //  in  Reaper struct that work on pointers whose lifetimes are not known. We should find ONE
    //  solution. Probably it's good to follow this: If we can guarantee UB, we should do it, if
    //  not,  we should mark the method unsafe. Is there any way to guarantee? I see this:
    //  a) Use something like the ValidatePtr function if available. However, calling it for each
    //     invocation is too presumptuous for an unopinionated medium-level API.
    //  b) Also store an ID or something (e.g. section ID here) and always refetch it. Same like
    //     with a ... very presumptuous.
    //  So none of this is really feasible on this API level. Which means that we must either rely
    //  on REAPER itself not running into UB ([] Try and askjf) or just mark the
    //  methods where this is not possible as unsafe. A higher-level API then should take care of
    //  making things absolutely safe.
    pub fn action_list_cnt(&self) -> u32 {
        unsafe { (*self.0).action_list_cnt as u32 }
    }

    pub fn get_action_by_index<'a>(&'a self, index: u32) -> Option<KbdCmd<'a>> {
        let array = unsafe {
            std::slice::from_raw_parts((*self.0).action_list, (*self.0).action_list_cnt as usize)
        };
        let raw_kbd_cmd = array.get(index as usize)?;
        Some(KbdCmd(raw_kbd_cmd))
    }
}

// There's no point in using references with lifetime annotations in `KbdSectionInfo` because it is
// impossible to track their lifetimes. However, we can start using lifetime annotations for
// KbdCmd because its lifetime can be related to the lifetime of the `KbdSectionInfo`.
pub struct KbdCmd<'a>(pub(super) &'a raw::KbdCmd);

impl<'a> KbdCmd<'a> {
    pub fn cmd(&self) -> u32 {
        self.0.cmd
    }
}
