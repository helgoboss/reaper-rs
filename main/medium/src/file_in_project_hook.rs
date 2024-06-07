use crate::{decode_user_data, encode_user_data, ReaProject, ReaperStr};
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
    inner: raw::file_in_project_ex2_t,
    callback: Box<dyn FileInProjectCallback>,
}

impl Debug for OwnedFileInProjectHook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // FileInProjectCallback doesn't generally implement Debug.
        f.debug_struct("OwnedFileInProjectHook")
            .field("inner", &self.inner)
            .field("callback", &"<omitted>")
            .finish()
    }
}

impl OwnedFileInProjectHook {
    pub fn new<T>(file_name: &ReaperStr, project: ReaProject, callback: Box<T>) -> Self
    where
        T: FileInProjectCallback + 'static,
    {
        Self {
            inner: raw::file_in_project_ex2_t {
                // It's okay that this file name goes out of memory. It's explicitly documented that it
                // only needs to be accessible at the time of registering the callback.
                file_name: file_name.as_ptr() as *mut _,
                proj_ptr: project.as_ptr(),
                user_data_context: encode_user_data(&callback),
                file_in_project_callback: Some(delegating_callback::<T>),
            },
            callback,
        }
    }

    pub fn into_callback(self) -> Box<dyn FileInProjectCallback> {
        self.callback
    }
}

impl AsRef<raw::file_in_project_ex2_t> for OwnedFileInProjectHook {
    fn as_ref(&self) -> &raw::file_in_project_ex2_t {
        &self.inner
    }
}

extern "C" fn delegating_callback<T: FileInProjectCallback>(
    user_data: *mut c_void,
    msg: c_int,
    parm: *mut c_void,
) -> INT_PTR {
    firewall(|| {
        let callback_struct: &mut T = decode_user_data(user_data);
        match msg {
            0x000 => {
                if parm.is_null() {
                    return 0;
                }
                let new_name = unsafe { ReaperStr::from_ptr(parm as _) };
                callback_struct.renamed(new_name);
                0
            }
            0x100 => callback_struct
                .get_directory_name()
                .map(|name| name.as_ptr() as _)
                .unwrap_or(0),
            _ => {
                let args = FileInProjectCallbackExtArgs { msg, parm };
                callback_struct.ext(args)
            }
        }
    })
    .unwrap_or(0)
}
