#![allow(non_snake_case)]
#![allow(clippy::wrong_self_convention)]

use crate::{ProjectStateContext, ReaperStr};
use reaper_low::{create_cpp_to_rust_project_state_context, raw};
use ref_cast::RefCast;
use std::fmt;
use std::os::raw::{c_char, c_int, c_longlong};
use std::ptr::NonNull;

/// Owned project state context.
///
/// This project state context automatically destroys the associated C++ `ProjectStateContext` when
/// dropped.
#[derive(Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedProjectStateContext(pub(crate) ProjectStateContext);

impl OwnedProjectStateContext {
    /// Takes ownership of the given context.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given context is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: ProjectStateContext) -> Self {
        Self(raw)
    }
}

unsafe impl Send for OwnedProjectStateContext {}

impl Drop for OwnedProjectStateContext {
    fn drop(&mut self) {
        unsafe {
            reaper_low::delete_cpp_project_state_context(self.0);
        }
    }
}

impl AsRef<BorrowedProjectStateContext> for OwnedProjectStateContext {
    fn as_ref(&self) -> &BorrowedProjectStateContext {
        BorrowedProjectStateContext::from_raw(unsafe { self.0.as_ref() })
    }
}

impl AsMut<BorrowedProjectStateContext> for OwnedProjectStateContext {
    fn as_mut(&mut self) -> &mut BorrowedProjectStateContext {
        BorrowedProjectStateContext::from_raw_mut(unsafe { self.0.as_mut() })
    }
}

/// Pointer to a project state context.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedProjectStateContext(raw::ProjectStateContext);

impl BorrowedProjectStateContext {
    /// Creates a medium-level representation from the given low-level reference.
    pub fn from_raw(raw: &raw::ProjectStateContext) -> &Self {
        Self::ref_cast(raw)
    }
    /// Creates a mutable medium-level representation from the given low-level reference.
    pub fn from_raw_mut(raw: &mut raw::ProjectStateContext) -> &mut Self {
        Self::ref_cast_mut(raw)
    }

    /// Returns the pointer to this context.
    pub fn as_ptr(&self) -> NonNull<raw::ProjectStateContext> {
        NonNull::from(&self.0)
    }
}

/// Consumers can implement this trait in order to provide own project state context types.
///
/// Attention: Not usable yet for writing due to the lack of the variadic parameter in AddLine.
pub trait CustomProjectStateContext {
    /// Writes the given line to this project state.
    ///
    /// Attention: Not usable yet due to the lack of the variadic parameter.
    fn add_line(&mut self, line: &ReaperStr);

    /// Obtains the next line from the project state by writing it to the given buffer.
    ///
    /// The line end is not marked with a newline character but a nul-byte!
    ///
    /// Returns `false` if no lines left.
    fn get_line(&mut self, line: &mut [c_char]) -> bool;

    /// Returns output size written so far.
    fn get_output_size(&mut self) -> u64;

    fn get_temp_flag(&mut self) -> i32;

    fn set_temp_flag(&mut self, flag: i32);
}

/// Represents an owned project state context that is backed by a Rust
/// [`CustomProjectStateContext`] trait implementation.
pub struct CustomOwnedProjectStateContext {
    // Those 2 belong together. `cpp_context` without `rust_context` = crash. Never let them apart!
    // TODO-high See Duplicate() of CustomPcmSource. We could actually let them apart and make
    //  the C++ destructor call a destroy function on Rust side. That would be more consequent.
    cpp_context: OwnedProjectStateContext,
    /// Never read but important to keep in memory.
    #[allow(clippy::redundant_allocation)]
    _rust_context: Box<Box<dyn reaper_low::ProjectStateContext>>,
}

impl fmt::Debug for CustomOwnedProjectStateContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CustomOwnedProjectStateContext")
            .field("cpp_context", &self.cpp_context)
            .finish()
    }
}

impl AsRef<BorrowedProjectStateContext> for CustomOwnedProjectStateContext {
    fn as_ref(&self) -> &BorrowedProjectStateContext {
        self.cpp_context.as_ref()
    }
}

impl AsMut<BorrowedProjectStateContext> for CustomOwnedProjectStateContext {
    fn as_mut(&mut self) -> &mut BorrowedProjectStateContext {
        self.cpp_context.as_mut()
    }
}

#[derive(Debug)]
struct ProjectStateContextAdapter<S: CustomProjectStateContext> {
    // See PcmSourceAdapter for further explanation.
    delegate: S,
}

impl<S: CustomProjectStateContext> ProjectStateContextAdapter<S> {
    pub fn new(delegate: S) -> Self {
        Self { delegate }
    }
}

impl<S: CustomProjectStateContext> reaper_low::ProjectStateContext
    for ProjectStateContextAdapter<S>
{
    fn AddLine(&mut self, line: *const c_char) {
        let line = unsafe { ReaperStr::from_ptr(line) };
        self.delegate.add_line(line);
    }

    fn GetLine(&mut self, buf: *mut c_char, buflen: c_int) -> c_int {
        let slice = unsafe { std::slice::from_raw_parts_mut(buf, buflen as usize) };
        if self.delegate.get_line(slice) {
            0
        } else {
            -1
        }
    }

    fn GetOutputSize(&mut self) -> c_longlong {
        self.delegate.get_output_size() as _
    }

    fn GetTempFlag(&mut self) -> c_int {
        self.delegate.get_temp_flag()
    }

    fn SetTempFlag(&mut self, flag: c_int) {
        self.delegate.set_temp_flag(flag);
    }
}

/// Unstable!!!
///
/// Creates a REAPER project state context for the given custom Rust implementation and returns it.
//
// TODO-high-unstable Think of a good name.
pub fn create_custom_owned_project_state_context<C: CustomProjectStateContext + 'static>(
    custom_context: C,
) -> CustomOwnedProjectStateContext {
    let adapter = ProjectStateContextAdapter::new(custom_context);
    // Create the C++ counterpart context (we need to box the Rust side twice in order to obtain
    // a thin pointer for passing it to C++ as callback target).
    let rust_context: Box<Box<dyn reaper_low::ProjectStateContext>> = Box::new(Box::new(adapter));
    let thin_ptr_to_adapter: NonNull<_> = rust_context.as_ref().into();
    let raw_cpp_context = unsafe { create_cpp_to_rust_project_state_context(thin_ptr_to_adapter) };
    let cpp_context = unsafe { OwnedProjectStateContext::from_raw(raw_cpp_context) };
    CustomOwnedProjectStateContext {
        cpp_context: cpp_context,
        _rust_context: rust_context,
    }
}
