use std::ffi::CStr;
use crate::high_level::{Project, Reaper};

// Constructor takes care of starting the undo block. Destructor takes care of ending the undo block (RAII).
// Doesn't start a new block if we already are in an undo block.
pub(super) struct UndoBlock<'a> {
    label: &'a CStr,
    project: Project,
}

impl UndoBlock<'_> {
    pub(super) fn new(project: Project, label: &CStr) -> UndoBlock {
        UndoBlock {
            label,
            project
        }
    }
}

impl Drop for UndoBlock<'_> {
    fn drop(&mut self) {
        Reaper::instance().leave_undo_block_internal(&self.project, self.label);
    }
}