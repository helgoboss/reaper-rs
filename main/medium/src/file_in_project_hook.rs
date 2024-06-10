use crate::{decode_user_data, encode_user_data, ReaProject, ReaperStr, ReaperString};
use reaper_low::raw::INT_PTR;
use reaper_low::{firewall, raw};
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::os::raw::c_int;

/// Consumers need to implement this trait in order to be called back if REAPER does something with a registered project
/// file.
///
/// See [`ReaperSession::plugin_register_add_file_in_project_ex2`].
pub trait FileInProjectCallback {
    /// This is called before REAPER renames files and allows you to determine in which subdirectory the
    /// file should go. Return `None` if you don't need the file to be in a subdirectory.
    fn get_directory_name(&mut self) -> Option<&'static ReaperStr> {
        None
    }

    /// File has been renamed.
    fn renamed(&mut self, new_name: &ReaperStr) {
        let _ = new_name;
    }

    /// *reaper-rs* calls this for unknown msg types. It's the fallback handler, so to say.
    fn ext(&mut self, args: FileInProjectCallbackExtArgs) -> INT_PTR {
        let _ = args;
        0
    }
}

pub struct FileInProjectCallbackExtArgs {
    pub msg: c_int,
    pub parm: *mut c_void,
}

pub(crate) struct OwnedFileInProjectHook {
    project: ReaProject,
    user_data: Box<UserData>,
}

impl Debug for OwnedFileInProjectHook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // FileInProjectCallback doesn't generally implement Debug.
        f.debug_struct("OwnedFileInProjectHook")
            .field("callback", &"<omitted>")
            .finish()
    }
}

struct UserData {
    file_name: ReaperString,
    callback: Box<dyn FileInProjectCallback>,
}

impl OwnedFileInProjectHook {
    pub fn new(
        project: ReaProject,
        file_name: ReaperString,
        callback: Box<dyn FileInProjectCallback>,
    ) -> Self {
        Self {
            project,
            user_data: Box::new(UserData {
                file_name,
                callback,
            }),
        }
    }

    /// This must be called when `self` is at its final place in memory!
    pub fn create_plugin_register_arg(&self) -> raw::file_in_project_ex2_t {
        raw::file_in_project_ex2_t {
            file_name: self.user_data.file_name.as_ptr() as *mut _,
            proj_ptr: self.project.as_ptr(),
            user_data_context: encode_user_data(&self.user_data),
            file_in_project_callback: Some(delegating_callback),
        }
    }
}

extern "C" fn delegating_callback(
    user_data: *mut c_void,
    msg: c_int,
    parm: *mut c_void,
) -> INT_PTR {
    firewall(|| {
        let user_data: &mut UserData = decode_user_data(user_data);
        match msg {
            0x000 => {
                if parm.is_null() {
                    return 0;
                }
                let new_name = unsafe { ReaperStr::from_ptr(parm as _) };
                // Update our own filename so that we can unregister correctly at a later point
                // (project pointer, file name content and user data pointer must match when unregistering)
                user_data.file_name = new_name.to_reaper_string();
                // Inform the consumer via callback about the rename so that it can react as well
                user_data.callback.renamed(new_name);
                0
            }
            0x100 => user_data
                .callback
                .get_directory_name()
                .map(|name| name.as_ptr() as _)
                .unwrap_or(0),
            _ => {
                let args = FileInProjectCallbackExtArgs { msg, parm };
                user_data.callback.ext(args)
            }
        }
    })
    .unwrap_or(0)
}
