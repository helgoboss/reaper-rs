use crate::raw::{ReaProject, INT_PTR};
use std::ffi::{c_char, c_int, c_void};

/// This structure is only documented, in <https://github.com/justinfrankel/reaper-sdk/blob/main/sdk/reaper_plugin.h>
/// (see "file_in_project_ex2").
///
/// It's documented as array but in accordance with all the other types we express it as struct with named fields.
/// The important thing is that the memory layout is the same, which it is because each field in the struct
/// is a pointer (with the same size).
///
/// **Keeping this particular field order is vital!**
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct file_in_project_ex2_t {
    pub file_name: *mut c_char,
    pub proj_ptr: *mut ReaProject,
    pub user_data_context: *mut c_void,
    pub file_in_project_callback: Option<
        unsafe extern "C" fn(user_data: *mut c_void, msg: c_int, param: *mut c_void) -> INT_PTR,
    >,
}
