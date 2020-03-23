use crate::high_level::{Project, Reaper};
use std::ffi::CStr;

// Constructor takes care of starting the undo block. Destructor takes care of ending the undo block (RAII).
pub(super) struct UndoBlock<'a> {
    label: &'a CStr,
    project: Project,
}

impl UndoBlock<'_> {
    pub(super) fn new(project: Project, label: &CStr) -> UndoBlock {
        UndoBlock { label, project }
    }
}

impl Drop for UndoBlock<'_> {
    fn drop(&mut self) {
        Reaper::get().leave_undo_block_internal(&self.project, self.label);
    }
}
